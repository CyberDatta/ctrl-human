<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { invoke } from '@tauri-apps/api/core';
  import '$lib/styles/tokens.css';
  import switchCardIcon from '$lib/assets/icons/switch_card.svg';

  const controllerId = $page.url.searchParams.get('id') ?? '';
  let controllerTitle = decodeURIComponent($page.url.searchParams.get('name') ?? 'Unknown Controller');
  const backUrl = decodeURIComponent($page.url.searchParams.get('back') ?? '') || `/controller-studio/controller-library/edit?id=${controllerId}`;
  const backText = decodeURIComponent($page.url.searchParams.get('backText') ?? '') || controllerTitle;
  const isMainController = $page.url.searchParams.get('back') === '/';

  // ── Controller pose rows (ordered, with priority) ──
  type ControllerPose = { idx: number; pose_id: string; priority: number; name: string; input_type: string; inputs: string[] };
  let controllerPoses: ControllerPose[] = [];

  // ── Testing data keyed by pose_id ──
  type PosePoint = { x: number; y: number };
  type TestingEntry = {
    pose_id: string;
    detection_method: 'cosine' | 'relative';
    confidence: number;
    active_landmarks: boolean[];
    reference_values: PosePoint[][];
  };
  let testingDataMap = new Map<string, TestingEntry>();

  // ── Which pose row indices are currently "selected" (highlighted) ──
  let selectedIdxs = new Set<number>();
  let prevSelectedIdxs = new Set<number>();

  // ── Input firing state ──
  const crazyTapIntervals = new Map<number, ReturnType<typeof setInterval>>();
  const CRAZY_TAP_INTERVAL_MS = 100;

  // ── Detection helpers (same as PoseTestingScreen) ──
  function cosineSimilarity(live: PosePoint[], ref: PosePoint[], mask: boolean[]): number {
    const liveA = live.filter((_, i) => mask[i]);
    const refA  = ref.filter((_, i) => mask[i]);
    if (liveA.length === 0) return 0;
    const liveXm = liveA.reduce((s, p) => s + p.x, 0) / liveA.length;
    const liveYm = liveA.reduce((s, p) => s + p.y, 0) / liveA.length;
    const refXm  = refA.reduce((s, p) => s + p.x, 0) / refA.length;
    const refYm  = refA.reduce((s, p) => s + p.y, 0) / refA.length;
    const liveVec = liveA.flatMap(p => [p.x - liveXm, p.y - liveYm]);
    const refVec  = refA.flatMap(p => [p.x - refXm,  p.y - refYm]);
    let dot = 0, magL = 0, magR = 0;
    for (let i = 0; i < liveVec.length; i++) {
      dot  += liveVec[i] * refVec[i];
      magL += liveVec[i] * liveVec[i];
      magR += refVec[i]  * refVec[i];
    }
    magL = Math.sqrt(magL); magR = Math.sqrt(magR);
    if (magL === 0 || magR === 0) return 0;
    return (dot / (magL * magR) + 1) / 2;
  }

  function rankOrder(values: number[]): number[] {
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
    return liveXRanks.every((r, i) => r === refXRanks[i]) &&
           liveYRanks.every((r, i) => r === refYRanks[i]);
  }

  function poseMatches(live: PosePoint[], entry: TestingEntry): boolean {
    if (entry.reference_values.length === 0) return false;
    if (entry.detection_method === 'cosine') {
      return entry.reference_values.some(ref =>
        cosineSimilarity(live, ref, entry.active_landmarks) >= entry.confidence
      );
    } else {
      return entry.reference_values.some(ref =>
        relativeMatch(live, ref, entry.active_landmarks)
      );
    }
  }

  function handleInputTransitions(newSelectedIdxs: Set<number>) {
    for (const cp of controllerPoses) {
      const wasSelected = prevSelectedIdxs.has(cp.idx);
      const isSelected = newSelectedIdxs.has(cp.idx);

      if (!wasSelected && isSelected) {
        // In-transition
        if (cp.input_type === 'Press') {
          invoke('fire_press', { inputs: cp.inputs }).catch(() => {});
        } else if (cp.input_type === 'Tap') {
          invoke('fire_tap', { inputs: cp.inputs }).catch(() => {});
        } else if (cp.input_type === 'Crazy Tap') {
          invoke('fire_tap', { inputs: cp.inputs }).catch(() => {});
          const interval = setInterval(() => {
            invoke('fire_tap', { inputs: cp.inputs }).catch(() => {});
          }, CRAZY_TAP_INTERVAL_MS);
          crazyTapIntervals.set(cp.idx, interval);
        }
      } else if (wasSelected && !isSelected) {
        // Out-transition
        if (cp.input_type === 'Press') {
          invoke('fire_release', { inputs: cp.inputs }).catch(() => {});
        } else if (cp.input_type === 'Crazy Tap') {
          const interval = crazyTapIntervals.get(cp.idx);
          if (interval !== undefined) {
            clearInterval(interval);
            crazyTapIntervals.delete(cp.idx);
          }
        }
      }
    }
    prevSelectedIdxs = new Set(newSelectedIdxs);
  }

  function runControllerDetection(live: PosePoint[]) {
    const matched = controllerPoses.filter(cp => {
      const entry = testingDataMap.get(cp.pose_id);
      return entry ? poseMatches(live, entry) : false;
    });

    const newSelectedIdxs = matched.length === 0
      ? new Set<number>()
      : new Set(
          matched
            .filter(cp => cp.priority === Math.min(...matched.map(c => c.priority)))
            .map(cp => cp.idx)
        );

    if (isMainController) {
      handleInputTransitions(newSelectedIdxs);
    }
    selectedIdxs = newSelectedIdxs;
  }

  // ── Skeleton overlay constants ──
  const POSE_CONNECTIONS: [number, number][] = [
    [0,1],[1,2],[2,3],[3,7],
    [0,4],[4,5],[5,6],[6,8],
    [9,10],
    [11,12],[11,23],[12,24],[23,24],
    [11,13],[13,15],[15,17],[17,19],[19,15],[15,21],
    [12,14],[14,16],[16,18],[18,20],[20,16],[16,22],
    [23,25],[25,27],[27,29],[29,31],[27,31],
    [24,26],[26,28],[28,30],[30,32],[28,32],
  ];
  const LEFT_LANDMARKS  = new Set([1,2,3,7,9,11,13,15,17,19,21,23,25,27,29,31]);
  const RIGHT_LANDMARKS = new Set([4,5,6,8,10,12,14,16,18,20,22,24,26,28,30,32]);

  // ── Camera ──
  let canvasEl: HTMLCanvasElement;
  let cameraError = '';
  let webcams: { index: number; name: string }[] = [];
  let currentWebcamPos = 0;
  let pollRunning = false;
  let pollGeneration = 0;
  let landmarks: { x: number; y: number }[] | null = null;

  function drawOverlay(ctx: CanvasRenderingContext2D) {
    if (!landmarks || landmarks.length < 33) return;
    const w = canvasEl.width;
    const h = canvasEl.height;

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
    for (let i = 0; i < landmarks.length; i++) {
      const lm = landmarks[i];
      ctx.beginPath();
      ctx.arc(lm.x * w, lm.y * h, 7, 0, Math.PI * 2);
      ctx.fillStyle = LEFT_LANDMARKS.has(i) ? '#00D9E8' : RIGHT_LANDMARKS.has(i) ? '#FF7043' : '#FFFFFF';
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
      } catch { /* ignore per-frame errors */ }
    }
  }

  async function pollLandmarks(landmarksUrl: string, generation: number) {
    while (pollRunning && pollGeneration === generation) {
      try {
        const response = await fetch(landmarksUrl);
        if (response.ok) {
          landmarks = await response.json();
          if (landmarks) runControllerDetection(landmarks);
        }
      } catch { /* ignore */ }
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
    pollRunning = false;
    invoke('stop_camera_stream').catch(() => {});
    goto(backUrl);
  }

  onMount(async () => {
    try { webcams = await invoke<{ index: number; name: string }[]>('list_webcams'); } catch { /* default */ }

    if (controllerId) {
      try {
        const [controller, poses] = await Promise.all([
          invoke<{ title: string; poses: { pose_id: string; priority: number; input_type?: string; input?: string[] }[] }>('load_controller', { controllerId }),
          invoke<{ pose_id: string; title: string }[]>('load_poses'),
        ]);
        controllerTitle = controller.title || 'Unknown Controller';
        const poseMap = new Map(poses.map(p => [p.pose_id, p.title]));
        controllerPoses = (controller.poses ?? []).map((p, i) => ({
          idx: i,
          pose_id: p.pose_id,
          priority: p.priority ?? 0,
          name: poseMap.get(p.pose_id) ?? 'Unknown Pose',
          input_type: p.input_type ?? 'Tap',
          inputs: p.input ?? [],
        }));
      } catch (e) { console.error('load_controller failed:', e); }

      try {
        const poseIds = controllerPoses.map(cp => cp.pose_id);
        const entries = await invoke<TestingEntry[]>('load_poses_for_testing', { poseIds });
        for (const entry of entries) {
          testingDataMap.set(entry.pose_id, entry);
        }
      } catch (e) { console.error('load_poses_for_testing failed:', e); }
    }

    const firstIndex = webcams.length > 0 ? webcams[0].index : 0;
    startCamera(firstIndex);
  });

  onDestroy(async () => {
    pollRunning = false;
    if (isMainController) {
      // Release any held Press inputs
      for (const cp of controllerPoses) {
        if (prevSelectedIdxs.has(cp.idx) && cp.input_type === 'Press') {
          invoke('fire_release', { inputs: cp.inputs }).catch(() => {});
        }
      }
      // Clear all crazy tap intervals
      for (const interval of crazyTapIntervals.values()) {
        clearInterval(interval);
      }
      crazyTapIntervals.clear();
    }
    try { await invoke('stop_camera_stream'); } catch { /* ignore */ }
  });
</script>

<div class="test-screen">

  <!-- Top nav -->
  <nav class="top-bar">
    <button class="back-btn" on:click={goBack}>
      <svg class="back-arrow" viewBox="0 0 33 40" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M11,5 L21,5 L21,19 L28,19 L26.083,23.923 L24.167,23.923 L20.333,31.308 L18.554,31.308 L17.778,35 L15.222,35 L14.583,31.308 L12.667,31.308 L9.472,23.923 L6.917,23.923 L5,19 L11,19 Z" />
      </svg>
      <span class="back-text">{backText}</span>
    </button>
  </nav>

  <!-- Camera + Switch Camera -->
  <div class="main-area">

    <div class="video-preview">
      <canvas bind:this={canvasEl} class="video-feed" width="1280" height="720"></canvas>
      {#if cameraError}
        <div class="camera-error">{cameraError}</div>
      {/if}
    </div>

    <div class="action-col">
      <button class="switch-btn" on:click={switchCamera}>
        <img src={switchCardIcon} alt="" class="switch-icon" />
        <span>Switch Camera</span>
      </button>
    </div>

  </div>

  <!-- Pose pill strip -->
  <div class="pose-strip-wrapper">
    <div class="pose-strip">
      {#each controllerPoses as cp (cp.idx)}
        <div class="pose-pill" class:selected={selectedIdxs.has(cp.idx)}>{cp.name}</div>
      {/each}
    </div>
  </div>

</div>

<style>
  .test-screen {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background-color: var(--color-primary-3);
    font-family: var(--font-primary);
    padding: 2.5rem 3rem 0 3rem;
    box-sizing: border-box;
  }

  /* ── Nav ── */
  .top-bar {
    flex-shrink: 0;
    margin-bottom: 3rem;
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
    height: 4rem;
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
    flex-shrink: 0;
  }

  /* ── Video ── */
  .video-preview {
    width: 56.25rem;
    flex-shrink: 0;
    background-color: var(--color-background);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    position: relative;
    overflow: hidden;
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

  /* ── Switch Camera button — positioned to the right of the video ── */
  .action-col {
    position: absolute;
    left: calc(50% + 56.25rem / 2 + 2rem);
    top: 50%;
    transform: translateY(-50%);
  }

  .switch-btn {
    display: flex;
    align-items: center;
    gap: 1rem;
    background-color: var(--color-primary-2);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H4);
    font-size: var(--font-size-H4);
    color: var(--color-dark-1);
    padding: 0.85rem 1.75rem;
    white-space: nowrap;
  }

  .switch-btn:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  .switch-btn:hover .switch-icon {
    filter: brightness(0) invert(1);
  }

  .switch-icon {
    width: 2.25rem;
    height: 2.25rem;
    flex-shrink: 0;
  }

  /* ── Pose strip ── */
  .pose-strip-wrapper {
    flex-shrink: 0;
    padding: 2rem 0 2rem 0;
    overflow: hidden;
  }

  .pose-strip {
    display: flex;
    gap: 1.25rem;
    overflow-x: auto;
    padding-bottom: 0.5rem;
    scrollbar-width: none;
  }

  .pose-strip::-webkit-scrollbar {
    display: none;
  }

  .pose-pill {
    flex-shrink: 0;
    background-color: var(--color-secondary-2);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    padding: 1.25rem 2.5rem;
    font-weight: var(--font-weight-H3);
    font-size: var(--font-size-H3);
    line-height: var(--line-height-H3);
    color: var(--color-dark-1);
    white-space: nowrap;
    transition: background-color 0.1s;
  }

  .pose-pill.selected {
    background-color: var(--color-primary-2);
  }
</style>
