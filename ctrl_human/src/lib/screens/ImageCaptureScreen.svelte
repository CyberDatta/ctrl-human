<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { invoke } from '@tauri-apps/api/core';
  import '$lib/styles/tokens.css';
  import cameraIcon from '$lib/assets/icons/camera.svg';
  import crossRedIcon from '$lib/assets/icons/cross.svg';
  import switchCardIcon from '$lib/assets/icons/switch_card.svg';
  import ecstaticIcon from '$lib/assets/icons/ecstatic.svg';

  let poseName = $page.url.searchParams.get('name') ?? 'Untitled Pose';
  let poseId = $page.url.searchParams.get('id');

  // ── Camera via single-frame polling ──
  // WebKitGTK (Tauri/Linux) errors on streaming responses (MJPEG / XHR arraybuffer).
  // Polling /frame with individual fetch() + arrayBuffer() calls works reliably.
  let canvasEl: HTMLCanvasElement;
  let cameraError = '';
  let webcams: { index: number; name: string }[] = [];
  let currentWebcamPos = 0;
  let pollRunning = false;
  let pollGeneration = 0;

  function drawJpeg(jpeg: Uint8Array<ArrayBuffer>) {
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
      })
      .catch(() => {});
  }

  async function pollLoop(frameUrl: string, generation: number) {
    while (pollRunning && pollGeneration === generation) {
      try {
        const response = await fetch(frameUrl);
        if (response.ok) {
          const buffer = await response.arrayBuffer();
          drawJpeg(new Uint8Array(buffer));
        }
        // 503 means camera not ready yet — just retry on the next tick
      } catch {
        // Network errors on individual frames are ignored; stop only if superseded or stopped
      }
    }
  }

  async function startCamera(index: number = 0) {
    pollRunning = false;
    pollGeneration++;
    try { await invoke('stop_camera_stream'); } catch { /* no existing stream */ }

    cameraError = '';

    try {
      const port = await invoke<number>('start_camera_stream', { cameraIndex: index, withInference: false });
      pollRunning = true;
      pollLoop(`http://127.0.0.1:${port}/frame`, pollGeneration);
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

  // ── Selfie countdown + capture ──
  let countdown: number | null = null;

  function startSelfieCountdown() {
    if (countdown !== null) return;
    countdown = 10;
    const interval = setInterval(async () => {
      if (countdown! > 0) {
        countdown!--;
      } else {
        clearInterval(interval);
        await captureFrame();
        countdown = null;
      }
    }, 1000);
  }

  async function captureFrame() {
    if (!poseId || !canvasEl) return;
    const dataUrl = canvasEl.toDataURL('image/png');
    const base64 = dataUrl.replace('data:image/png;base64,', '');
    try {
      const imageId = await invoke<string>('save_pose_image', { poseId, imageData: base64 });
      captures = [...captures, { imageId, dataUrl }];
    } catch (err: unknown) {
      const msg = String(err);
      if (!msg.includes('No pose detected')) {
        console.error('Failed to save image:', err);
      }
    }
  }

  // ── Captures ──
  let captures: { imageId: string; dataUrl: string }[] = [];

  async function removeCapture(imageId: string) {
    if (!poseId) return;
    try {
      await invoke('delete_pose_image', { poseId, imageId });
    } catch (err) {
      console.error('Failed to delete image:', err);
    }
    captures = captures.filter(c => c.imageId !== imageId);
  }

  function goBack() {
    goto(poseId ? `/controller-studio/pose-library/edit?id=${poseId}` : '/controller-studio/pose-library');
  }

  onMount(async () => {
    // Get camera count for switch button
    try {
      webcams = await invoke<{ index: number; name: string }[]>('list_webcams');
    } catch { /* default: empty, startCamera(0) used directly */ }

    const firstIndex = webcams.length > 0 ? webcams[0].index : 0;
    startCamera(firstIndex);
  });

  onDestroy(async () => {
    pollRunning = false;
    try { await invoke('stop_camera_stream'); } catch { /* ignore */ }
  });
</script>

<div class="capture-screen">

  <!-- Top nav -->
  <nav class="top-bar">
    <button class="back-btn" on:click={goBack}>
      <svg class="back-arrow" viewBox="0 0 33 40" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M11,5 L21,5 L21,19 L28,19 L26.083,23.923 L24.167,23.923 L20.333,31.308 L18.554,31.308 L17.778,35 L15.222,35 L14.583,31.308 L12.667,31.308 L9.472,23.923 L6.917,23.923 L5,19 L11,19 Z" />
      </svg>
      <span class="back-text">{poseName}</span>
    </button>
  </nav>

  <!-- Main area: video preview + action buttons -->
  <div class="main-area">

    <!-- Camera preview via canvas -->
    <div class="video-preview">
      <canvas bind:this={canvasEl} class="video-feed" width="1280" height="720"></canvas>
      {#if countdown !== null}
        <div class="countdown-overlay">{countdown}</div>
      {/if}
      {#if cameraError}
        <div class="camera-error">{cameraError}</div>
      {/if}
    </div>

    <!-- Action buttons -->
    <div class="action-col">
      <button class="action-btn selfie-btn" on:click={startSelfieCountdown} disabled={countdown !== null}>
        <img src={cameraIcon} alt="" class="action-icon" />
        <span>Selfie Time</span>
      </button>

      <button
        class="action-btn switch-btn"
        on:click={switchCamera}
      >
        <img src={switchCardIcon} alt="" class="action-icon" />
        <span>Switch Camera</span>
      </button>

      <button class="action-btn over-btn" on:click={goBack}>
        <img src={ecstaticIcon} alt="" class="action-icon" />
        <span>Selfie Time Over</span>
      </button>
    </div>
  </div>

  <!-- Horizontally scrollable captures strip -->
  <div class="captures-strip">
    {#each captures as cap (cap.imageId)}
      <div class="capture-card">
        <img src={cap.dataUrl} alt="Captured pose" class="capture-img" />
        <button class="remove-btn" on:click={() => removeCapture(cap.imageId)} aria-label="Remove capture">
          <img src={crossRedIcon} alt="Remove" class="remove-icon" />
        </button>
      </div>
    {/each}
  </div>

</div>

<style>
  .capture-screen {
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
    display: flex;
    gap: 2vw;
    align-items: center;
    justify-content: center;
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

  .countdown-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-primary);
    font-size: 20rem;
    font-weight: var(--font-weight-Huge);
    color: var(--color-background);
    -webkit-text-stroke: 0.4rem var(--color-dark-1);
    pointer-events: none;
    user-select: none;
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

  /* ── Action buttons column ── */
  .action-col {
    flex-shrink: 0;
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
    transition: background-color 0.1s, color 0.1s;
    width: 22vw;
  }

  .action-btn:hover:not(:disabled) {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  .action-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
    box-shadow: none;
  }

  .selfie-btn  { background-color: var(--color-secondary-2); }
  .switch-btn  { background-color: var(--color-primary-2); }
  .over-btn    { background-color: var(--color-secondary-4); }

  .action-icon {
    width: 2.25rem;
    height: 2.25rem;
    flex-shrink: 0;
  }

  /* ── Captures strip ── */
  .captures-strip {
    flex-shrink: 0;
    display: flex;
    flex-direction: row;
    gap: 1.5vw;
    overflow-x: auto;
    padding: 2vh 3.5vw;
    margin: 4vh -3.5vw 0;
    scrollbar-width: thin;
    scrollbar-color: var(--color-dark-1) transparent;
  }

  .captures-strip::-webkit-scrollbar { height: 0.4rem; }
  .captures-strip::-webkit-scrollbar-thumb {
    background-color: var(--color-dark-1);
    border-radius: 999px;
  }

  .capture-card {
    flex-shrink: 0;
    width: 19vw;
    height: 20vh;
    background-color: var(--color-background);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    position: relative;
    overflow: hidden;
  }

  .capture-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .remove-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    position: absolute;
    top: 0.25rem;
    left: 0.25rem;
  }

  .remove-btn:hover { opacity: 0.75; }

  .remove-icon {
    width: 2rem;
    height: 2rem;
  }
</style>
