<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { invoke } from '@tauri-apps/api/core';
  import '$lib/styles/tokens.css';
  import switchCardIcon from '$lib/assets/icons/switch_card.svg';

  let poseName = $page.url.searchParams.get('name') ?? 'Untitled Pose';
  let poseId = $page.url.searchParams.get('id');

  // ── Pose detection data loaded from save file ──
  type PosePoint = { x: number; y: number };
  type TestingData = {
    detection_method: 'cosine' | 'relative';
    confidence: number;
    active_landmarks: boolean[];
    reference_values: PosePoint[][];
  };
  let testingData: TestingData | null = null;

  // ── Detection logic ──

  function cosineSimilarity(live: PosePoint[], ref: PosePoint[], mask: boolean[]): number {
    // Filter to active landmarks only
    const liveA = live.filter((_, i) => mask[i]);
    const refA  = ref.filter((_, i) => mask[i]);
    if (liveA.length === 0) return 0;

    // Center-normalize: subtract mean so position in frame doesn't affect result
    const liveXm = liveA.reduce((s, p) => s + p.x, 0) / liveA.length;
    const liveYm = liveA.reduce((s, p) => s + p.y, 0) / liveA.length;
    const refXm  = refA.reduce((s, p)  => s + p.x, 0) / refA.length;
    const refYm  = refA.reduce((s, p)  => s + p.y, 0) / refA.length;

    const liveVec = liveA.flatMap(p => [p.x - liveXm, p.y - liveYm]);
    const refVec  = refA.flatMap(p  => [p.x - refXm,  p.y - refYm]);

    let dot = 0, magL = 0, magR = 0;
    for (let i = 0; i < liveVec.length; i++) {
      dot  += liveVec[i] * refVec[i];
      magL += liveVec[i] * liveVec[i];
      magR += refVec[i]  * refVec[i];
    }
    magL = Math.sqrt(magL);
    magR = Math.sqrt(magR);
    if (magL === 0 || magR === 0) return 0;

    const cosSim = dot / (magL * magR); // -1 to 1
    return (cosSim + 1) / 2;            // 0 to 1
  }

  function rankOrder(values: number[]): number[] {
    // Returns the rank (0 = smallest) of each element in the input array
    const indexed = values.map((v, i): [number, number] => [v, i]);
    indexed.sort((a, b) => a[0] - b[0]);
    const ranks = new Array<number>(values.length);
    indexed.forEach(([, origIdx], rank) => { ranks[origIdx] = rank; });
    return ranks;
  }

  function relativeMatch(live: PosePoint[], ref: PosePoint[], mask: boolean[]): boolean {
    const liveA = live.filter((_, i) => mask[i]);
    const refA  = ref.filter((_, i) => mask[i]);
    if (liveA.length === 0) return false;

    const liveXRanks = rankOrder(liveA.map(p => p.x));
    const liveYRanks = rankOrder(liveA.map(p => p.y));
    const refXRanks  = rankOrder(refA.map(p => p.x));
    const refYRanks  = rankOrder(refA.map(p => p.y));

    const xMatch = liveXRanks.every((r, i) => r === refXRanks[i]);
    const yMatch = liveYRanks.every((r, i) => r === refYRanks[i]);
    return xMatch && yMatch;
  }

  function checkDetection(live: PosePoint[]): boolean {
    if (!testingData || testingData.reference_values.length === 0) return false;
    const { detection_method, confidence, active_landmarks, reference_values } = testingData;

    if (detection_method === 'cosine') {
      return reference_values.some(ref =>
        cosineSimilarity(live, ref, active_landmarks) >= confidence
      );
    } else {
      return reference_values.some(ref =>
        relativeMatch(live, ref, active_landmarks)
      );
    }
  }

  // ── Skeleton overlay constants ──
  // Standard MediaPipe Pose connections
  const POSE_CONNECTIONS: [number, number][] = [
    // Face
    [0,1],[1,2],[2,3],[3,7],
    [0,4],[4,5],[5,6],[6,8],
    [9,10],
    // Torso
    [11,12],[11,23],[12,24],[23,24],
    // Left arm
    [11,13],[13,15],[15,17],[17,19],[19,15],[15,21],
    // Right arm
    [12,14],[14,16],[16,18],[18,20],[20,16],[16,22],
    // Left leg
    [23,25],[25,27],[27,29],[29,31],[27,31],
    // Right leg
    [24,26],[26,28],[28,30],[30,32],[28,32],
  ];

  // Person's left-body landmarks → cyan; right-body → orange
  const LEFT_LANDMARKS  = new Set([1,2,3,7,9,11,13,15,17,19,21,23,25,27,29,31]);
  const RIGHT_LANDMARKS = new Set([4,5,6,8,10,12,14,16,18,20,22,24,26,28,30,32]);

  // ── Camera via single-frame polling ──
  let canvasEl: HTMLCanvasElement;
  let cameraError = '';
  let webcams: { index: number; name: string }[] = [];
  let currentWebcamPos = 0;
  let pollRunning = false;
  let pollGeneration = 0;

  // ── Live landmarks from inference thread ──
  let landmarks: { x: number; y: number }[] | null = null;

  function drawOverlay(ctx: CanvasRenderingContext2D) {
    if (!landmarks || landmarks.length < 33) return;
    const w = canvasEl.width;
    const h = canvasEl.height;

    // Draw skeleton lines
    ctx.save();
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.9)';
    ctx.lineWidth = 3;
    ctx.lineCap = 'round';
    for (const [a, b] of POSE_CONNECTIONS) {
      const p1 = landmarks[a];
      const p2 = landmarks[b];
      ctx.beginPath();
      ctx.moveTo(p1.x * w, p1.y * h);
      ctx.lineTo(p2.x * w, p2.y * h);
      ctx.stroke();
    }

    // Draw landmark dots
    for (let i = 0; i < landmarks.length; i++) {
      const lm = landmarks[i];
      const x = lm.x * w;
      const y = lm.y * h;
      ctx.beginPath();
      ctx.arc(x, y, 7, 0, Math.PI * 2);
      ctx.fillStyle = LEFT_LANDMARKS.has(i)
        ? '#00D9E8'   // cyan for person's left side
        : RIGHT_LANDMARKS.has(i)
        ? '#FF7043'   // orange for person's right side
        : '#FFFFFF';  // white for center (nose)
      ctx.fill();
      ctx.strokeStyle = 'rgba(0,0,0,0.6)';
      ctx.lineWidth = 1.5;
      ctx.stroke();
    }
    ctx.restore();
  }

  function drawFrame(jpeg: Uint8Array<ArrayBuffer>) {
    const ctx = canvasEl?.getContext('2d');
    if (!ctx) return;
    createImageBitmap(new Blob([jpeg], { type: 'image/jpeg' }))
      .then(bitmap => {
        if (canvasEl.width !== bitmap.width || canvasEl.height !== bitmap.height) {
          canvasEl.width = bitmap.width;
          canvasEl.height = bitmap.height;
        }
        ctx.drawImage(bitmap, 0, 0);
        bitmap.close();
        drawOverlay(ctx);
      })
      .catch(() => {});
  }

  async function pollLoop(frameUrl: string, generation: number) {
    while (pollRunning && pollGeneration === generation) {
      try {
        const response = await fetch(frameUrl);
        if (response.ok) {
          const buffer = await response.arrayBuffer();
          drawFrame(new Uint8Array(buffer));
        }
      } catch {
        // ignore per-frame errors
      }
    }
  }

  async function pollLandmarks(landmarksUrl: string, generation: number) {
    while (pollRunning && pollGeneration === generation) {
      try {
        const response = await fetch(landmarksUrl);
        if (response.ok) {
          landmarks = await response.json();
          poseDetected = landmarks ? checkDetection(landmarks) : false;
        }
      } catch {
        // ignore per-poll errors
      }
    }
  }

  async function startCamera(index: number = 0) {
    pollRunning = false;
    pollGeneration++;
    landmarks = null;
    try { await invoke('stop_camera_stream'); } catch { /* no existing stream */ }

    cameraError = '';

    try {
      const port = await invoke<number>('start_camera_stream', { cameraIndex: index, withInference: true });
      const baseUrl = `http://127.0.0.1:${port}`;
      pollRunning = true;
      const gen = pollGeneration;
      pollLoop(`${baseUrl}/frame`, gen);
      pollLandmarks(`${baseUrl}/landmarks`, gen);
    } catch (err) {
      cameraError = String(err);
    }
  }

  async function switchCamera() {
    const count = webcams.length > 0 ? webcams.length : 10;
    currentWebcamPos = (currentWebcamPos + 1) % count;
    const idx = webcams.length > 0 ? webcams[currentWebcamPos].index : currentWebcamPos;
    await startCamera(idx);
  }

  function goBack() {
    goto(poseId ? `/controller-studio/pose-library/edit?id=${poseId}` : '/controller-studio/pose-library');
  }

  onMount(async () => {
    try {
      webcams = await invoke<{ index: number; name: string }[]>('list_webcams');
    } catch { /* default */ }

    if (poseId) {
      try {
        testingData = await invoke<TestingData>('load_pose_for_testing', { poseId });
      } catch { /* no testing data — detection disabled */ }
    }

    const firstIndex = webcams.length > 0 ? webcams[0].index : 0;
    startCamera(firstIndex);
  });

  onDestroy(async () => {
    pollRunning = false;
    try { await invoke('stop_camera_stream'); } catch { /* ignore */ }
  });

  let poseDetected = false;
</script>

<div class="test-screen">

  <!-- Top nav -->
  <nav class="top-bar">
    <button class="back-btn" on:click={goBack}>
      <svg class="back-arrow" viewBox="0 0 33 40" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M11,5 L21,5 L21,19 L28,19 L26.083,23.923 L24.167,23.923 L20.333,31.308 L18.554,31.308 L17.778,35 L15.222,35 L14.583,31.308 L12.667,31.308 L9.472,23.923 L6.917,23.923 L5,19 L11,19 Z" />
      </svg>
      <span class="back-text">{poseName}</span>
    </button>
  </nav>

  <!-- Main area -->
  <div class="main-area">

    <!-- Camera preview -->
    <div class="video-preview">
      <canvas bind:this={canvasEl} class="video-feed" width="1280" height="720"></canvas>
      {#if cameraError}
        <div class="camera-error">{cameraError}</div>
      {/if}
    </div>

    <!-- Side actions -->
    <div class="action-col">
      <button
        class="action-btn switch-btn"
        on:click={switchCamera}
      >
        <img src={switchCardIcon} alt="" class="action-icon" />
        <span>Switch Camera</span>
      </button>
    </div>

  </div>

  <!-- Detection status -->
  <div class="status-row">
    <div class="status-badge" class:detected={poseDetected} class:not-detected={!poseDetected}>
      {poseDetected ? 'Pose Detected' : 'Pose Not Detected'}
    </div>
  </div>

</div>

<style>
  .test-screen {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow-y: auto;
    background-color: var(--color-primary-3);
    font-family: var(--font-primary);
    padding: 3vh 3.5vw 0 3.5vw;
    box-sizing: border-box;
  }

  /* ── Nav ── */
  .top-bar {
    flex-shrink: 0;
    margin-bottom: 5vh;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 2rem;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .back-arrow {
    height: 5vh;
    transform: rotate(90deg);
    flex-shrink: 0;
  }

  .back-arrow path {
    fill: var(--color-background);
    stroke: var(--color-primary-outline);
    stroke-width: var(--stroke-width-xs);
    vector-effect: non-scaling-stroke;
  }

  .back-text {
    font-weight: var(--font-weight-H1);
    font-size: var(--font-size-H1);
    line-height: var(--line-height-H1);
    color: var(--color-background);
    -webkit-text-stroke: var(--stroke-width-xs) var(--color-primary-outline);
  }

  /* ── Main area ── */
  .main-area {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  /* ── Camera preview ── */
  .video-preview {
    width: 48vw;
    background-color: var(--color-background);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    position: relative;
    overflow: hidden;
    flex-shrink: 0;
  }

  .video-feed {
    width: 100%;
    height: auto;
    display: block;
  }

  .camera-error {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-background);
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H5);
    font-size: var(--font-size-H5);
    color: var(--color-dark-1);
    padding: 2rem;
    text-align: center;
  }

  /* ── Action column — sits to the right of the centered video ── */
  .action-col {
    position: absolute;
    left: calc(50% + 24vw + 2vw);
    top: 50%;
    transform: translateY(-50%);
    display: flex;
    flex-direction: column;
    gap: 1.5vh;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H4);
    font-size: var(--font-size-H4);
    color: var(--color-dark-1);
    padding: 0.85rem 1.5rem;
    width: 22vw;
  }

  .action-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
    box-shadow: none;
  }

  .switch-btn { background-color: var(--color-primary-2); }

  .action-icon {
    width: 2.25rem;
    height: 2.25rem;
    flex-shrink: 0;
  }

  /* ── Status row ── */
  .status-row {
    display: flex;
    justify-content: center;
    margin-top: 4vh;
  }

  .status-badge {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H3);
    font-size: var(--font-size-H3);
    color: var(--color-dark-1);
    padding: 0.75rem 3vw;
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    transition: background-color 0.2s;
  }

  .not-detected {
    background-color: var(--color-secondary-1);
  }

  .detected {
    background-color: var(--color-primary-2);
  }
</style>
