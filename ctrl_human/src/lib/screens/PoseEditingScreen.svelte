<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { invoke } from '@tauri-apps/api/core';
  import '$lib/styles/tokens.css';
  import pencilIcon from '$lib/assets/icons/pencil.svg';
  import cameraIcon from '$lib/assets/icons/camera.svg';
  import imageIcon from '$lib/assets/icons/image.svg';
  import labFlaskIcon from '$lib/assets/icons/lab_flask.svg';
  import mediapipeSkeleton from '$lib/assets/images/mediapipe_skeleton.svg';
  import downloadIcon from '$lib/assets/icons/arrow_for_download.svg';
  import trashIcon from '$lib/assets/icons/trash_can.svg';
  import checkedIcon from '$lib/assets/icons/checked.svg';
  import uncheckedIcon from '$lib/assets/icons/unchecked.svg';
  import crossRedIcon from '$lib/assets/icons/cross.svg';

  function goBack() {
    goto('/controller-studio/pose-library');
  }

  // ── Load pose from save file ──
  type DetectionMethod = 'cosine' | 'relative';
  let poseId: string | null = null;

  async function loadPose() {
    poseId = $page.url.searchParams.get('id');
    if (!poseId) return;
    const pose = await invoke<{
      title: string;
      description: string;
      detection: { method: string; confidence: number };
      active_landmarks: boolean[];
    }>('load_pose', { poseId });

    poseName = pose.title;
    description = pose.description;
    detectionMethod = (pose.detection.method === 'cosine' ? 'cosine' : 'relative') as DetectionMethod;
    minSimilarity = Math.round(pose.detection.confidence * 100);
    joints = joints.map((j, i) => ({ ...j, active: pose.active_landmarks[i] ?? true }));
  }

  function savePose() {
    if (!poseId) return;
    invoke('save_pose', {
      poseId,
      title: poseName,
      description,
      detectionMethod,
      confidence: minSimilarity / 100,
      activeLandmarks: joints.map(j => j.active),
    });
  }

  // ── Title editing ──
  let poseName = 'Untitled Pose 7';
  let editingTitle = false;
  let titleDraft = '';
  function startEditTitle() { titleDraft = poseName; editingTitle = true; }
  function commitTitle() { if (titleDraft.trim()) poseName = titleDraft.trim(); editingTitle = false; savePose(); }
  function onTitleKey(e: KeyboardEvent) {
    if (e.key === 'Enter') commitTitle();
    if (e.key === 'Escape') editingTitle = false;
  }

  // ── Description editing ──
  let description = 'This is a sample description of what this project does. This is a sample description of what this project does. This is a sample description of what this project does.';
  let editingDesc = false;
  let descDraft = '';
  function startEditDesc() { descDraft = description; editingDesc = true; }
  function commitDesc() { if (descDraft.trim()) description = descDraft.trim(); editingDesc = false; savePose(); }

  // ── Detection method ──
  let detectionMethod: DetectionMethod = 'relative';
  let minSimilarity = 50;

  // ── Slider debounce ──
  let sliderTimer: ReturnType<typeof setTimeout>;
  function onSliderChange() {
    clearTimeout(sliderTimer);
    sliderTimer = setTimeout(savePose, 400);
  }

  // ── Skeleton zoom/pan ──
  let viewportEl: HTMLDivElement;
  let viewScale = 1;
  let viewX = 0;
  let viewY = 0;
  let panning = false;
  let panAnchorX = 0;
  let panAnchorY = 0;
  let panBaseX = 0;
  let panBaseY = 0;
  let didPan = false;

  onMount(() => {
    viewportEl.addEventListener('wheel', onViewWheel, { passive: false });
    loadPose().then(() => loadPoseImages());
    return () => viewportEl.removeEventListener('wheel', onViewWheel);
  });

  function onViewWheel(e: WheelEvent) {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.12 : 0.89;
    viewScale = Math.max(0.15, Math.min(6, viewScale * factor));
  }

  function onViewMouseDown(e: MouseEvent) {
    panning = true;
    didPan = false;
    panAnchorX = e.clientX;
    panAnchorY = e.clientY;
    panBaseX = viewX;
    panBaseY = viewY;
  }

  function onViewMouseMove(e: MouseEvent) {
    if (!panning) return;
    const dx = e.clientX - panAnchorX;
    const dy = e.clientY - panAnchorY;
    if (Math.abs(dx) > 3 || Math.abs(dy) > 3) didPan = true;
    if (didPan) {
      viewX = panBaseX + dx;
      viewY = panBaseY + dy;
    }
  }

  function onViewMouseUp() { panning = false; }

  // ── Joints — all 33 MediaPipe Pose landmarks ──
  // Coordinates are in the mediapipe_skeleton.svg coordinate space (1448×1674).
  // cbdx/cbdy offsets position the checkbox relative to the joint centre.
  // Left-side joints (x < 724): checkbox to the left  (cbdx = -74)
  // Right-side joints (x ≥ 724): checkbox to the right (cbdx =  22)
  // cbdy = -25 centres the 50×50 checkbox vertically on the joint dot.
  type Joint = { id: string; x: number; y: number; cbdx: number; cbdy: number; active: boolean };
  let joints: Joint[] = [
    // ── Face ──
    { id: 'nose',             x: 725,  y: 252,  cbdx:  22, cbdy: -25, active: true  },
    { id: 'left_eye_inner',   x: 679,  y: 173,  cbdx:  -25, cbdy: -80, active: true  },
    { id: 'left_eye',         x: 646,  y: 171,  cbdx: -50, cbdy: -80, active: true  },
    { id: 'left_eye_outer',   x: 613,  y: 172,  cbdx: -75, cbdy: -80, active: true  },
    { id: 'right_eye_inner',  x: 775,  y: 175,  cbdx: -25, cbdy: -80, active: true  },
    { id: 'right_eye',        x: 809,  y: 175,  cbdx:  0, cbdy: -80, active: true  },
    { id: 'right_eye_outer',  x: 841,  y: 175,  cbdx:  25, cbdy: -80, active: true  },
    { id: 'left_ear',         x: 572,  y: 211,  cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_ear',        x: 892,  y: 215,  cbdx:  22, cbdy: -25, active: true  },
    { id: 'mouth_left',       x: 680,  y: 317,  cbdx: -74, cbdy: -25, active: true  },
    { id: 'mouth_right',      x: 775,  y: 317,  cbdx:  22, cbdy: -25, active: true  },
    // ── Upper body ──
    { id: 'left_shoulder',    x: 500,  y: 474,  cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_shoulder',   x: 951,  y: 474,  cbdx:  22, cbdy: -25, active: true  },
    { id: 'left_elbow',       x: 347,  y: 616,  cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_elbow',      x: 1099, y: 624,  cbdx:  22, cbdy: -25, active: true  },
    { id: 'left_wrist',       x: 203,  y: 548,  cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_wrist',      x: 1246, y: 558,  cbdx:  22, cbdy: -25, active: true  },
    { id: 'left_pinky',       x: 126,  y: 450,  cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_pinky',      x: 1325, y: 449,  cbdx:  22, cbdy: -25, active: true  },
    { id: 'left_index',       x: 82,   y: 557,  cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_index',      x: 1366, y: 548,  cbdx:  22, cbdy: -25, active: true  },
    { id: 'left_thumb',       x: 213,  y: 482,  cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_thumb',      x: 1243, y: 485,  cbdx:  22, cbdy: -25, active: true  },
    // ── Lower body ──
    { id: 'left_hip',         x: 568,  y: 878,  cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_hip',        x: 877,  y: 879,  cbdx:  22, cbdy: -25, active: true  },
    { id: 'left_knee',        x: 489,  y: 1203, cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_knee',       x: 951,  y: 1203, cbdx:  22, cbdy: -25, active: true  },
    { id: 'left_ankle',       x: 565,  y: 1528, cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_ankle',      x: 870,  y: 1525, cbdx:  22, cbdy: -25, active: true  },
    { id: 'left_heel',        x: 613,  y: 1612, cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_heel',       x: 822,  y: 1608, cbdx:  22, cbdy: -25, active: true  },
    { id: 'left_foot_index',  x: 447,  y: 1629, cbdx: -74, cbdy: -25, active: true  },
    { id: 'right_foot_index', x: 988,  y: 1627, cbdx:  22, cbdy: -25, active: true  },
  ];

  function toggleJoint(id: string, e: MouseEvent) {
    e.stopPropagation();
    joints = joints.map(j => j.id === id ? { ...j, active: !j.active } : j);
    savePose();
  }

  // ── Upload image ──
  let fileInputEl: HTMLInputElement;
  let uploading = false;

  async function handleFileUpload(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file || !poseId) return;
    uploading = true;

    const dataUrl = await new Promise<string>((resolve) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result as string);
      reader.readAsDataURL(file);
    });

    const base64 = dataUrl.replace('data:image/png;base64,', '');

    try {
      const imageId = await invoke<string>('save_pose_image', { poseId, imageData: base64 });
      refImages = [...refImages, { image_id: imageId, active: false, data_b64: base64 }];
    } catch (err: unknown) {
      if (!String(err).includes('No pose detected')) {
        console.error('Failed to upload image:', err);
      }
    } finally {
      uploading = false;
      fileInputEl.value = '';
    }
  }

  // ── Reference images ──
  type RefImage = { image_id: string; active: boolean; data_b64: string };
  let refImages: RefImage[] = [];

  async function loadPoseImages() {
    if (!poseId) return;
    try {
      refImages = await invoke<RefImage[]>('load_pose_images', { poseId });
    } catch { /* no images yet */ }
  }

  async function toggleRef(image_id: string) {
    const img = refImages.find(i => i.image_id === image_id);
    if (!img || !poseId) return;
    const newActive = !img.active;
    refImages = refImages.map(i => i.image_id === image_id ? { ...i, active: newActive } : i);
    invoke('set_image_active', { poseId, imageId: image_id, active: newActive });
  }

  async function removeRef(image_id: string) {
    if (!poseId) return;
    await invoke('delete_pose_image', { poseId, imageId: image_id });
    refImages = refImages.filter(i => i.image_id !== image_id);
  }
</script>

<div class="pose-editor">

  <!-- ── Left main panel ── -->
  <div class="main-panel">

    <!-- Nav -->
    <nav class="top-bar">
      <button class="back-btn" on:click={goBack}>
        <svg class="back-arrow" viewBox="0 0 33 40" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M11,5 L21,5 L21,19 L28,19 L26.083,23.923 L24.167,23.923 L20.333,31.308 L18.554,31.308 L17.778,35 L15.222,35 L14.583,31.308 L12.667,31.308 L9.472,23.923 L6.917,23.923 L5,19 L11,19 Z" />
        </svg>
        <span class="back-text">Pose Library</span>
      </button>
    </nav>

    <!-- Title -->
    <div class="title-row">
      {#if editingTitle}
        <input
          class="title-input"
          bind:value={titleDraft}
          on:blur={commitTitle}
          on:keydown={onTitleKey}
          autofocus
        />
      {:else}
        <span class="title-wrap">
          <h1 class="pose-title">{poseName}</h1>
          <button class="pencil-btn" on:click={startEditTitle} aria-label="Edit title">
            <img src={pencilIcon} alt="" class="pencil-icon" />
          </button>
        </span>
      {/if}
    </div>

    <!-- Description -->
    <div class="desc-row">
      {#if editingDesc}
        <textarea
          class="desc-input"
          bind:value={descDraft}
          on:blur={commitDesc}
          autofocus
        ></textarea>
      {:else}
        <p class="pose-desc">{description}</p>
        <button class="pencil-btn pencil-btn-desc" on:click={startEditDesc} aria-label="Edit description">
          <img src={pencilIcon} alt="" class="pencil-icon" />
        </button>
      {/if}
    </div>

    <!-- Editor area: skeleton viewport + controls -->
    <div class="editor-area">

      <!-- Skeleton viewport (zoomable / pannable) -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div
        class="skeleton-viewport"
        class:panning
        bind:this={viewportEl}
        on:mousedown={onViewMouseDown}
        on:mousemove={onViewMouseMove}
        on:mouseup={onViewMouseUp}
        on:mouseleave={onViewMouseUp}
      >
        <div
          class="skeleton-inner"
          style="transform: translate(calc(-50% + {viewX}px), calc(-50% + {viewY}px)) scale({viewScale});"
        >
          <!-- viewBox matches mediapipe_skeleton.svg (1448×1674); display at ~380×440 -->
          <svg width="380" height="440" viewBox="0 0 1448 1674" xmlns="http://www.w3.org/2000/svg">

            <!-- Mediapipe skeleton -->
            <image href={mediapipeSkeleton} width="1448" height="1674" />

            <!-- Checkbox overlays for each landmark -->
            {#each joints as joint (joint.id)}
              <!-- svelte-ignore a11y-click-events-have-key-events -->
              <!-- svelte-ignore a11y-no-static-element-interactions -->
              <g
                transform="translate({joint.x + joint.cbdx}, {joint.y + joint.cbdy})"
                on:mousedown|stopPropagation
                on:click={(e) => toggleJoint(joint.id, e)}
                style="cursor: pointer;"
              >
                <rect
                  x="0" y="0" width="50" height="50"
                  fill={joint.active ? 'var(--color-dark-1)' : 'var(--color-background)'}
                  stroke="var(--color-dark-1)"
                  stroke-width="4"
                />
                {#if joint.active}
                  <polyline
                    points="9,25 20,37 41,12"
                    fill="none"
                    stroke="white"
                    stroke-width="7"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                {/if}
              </g>
            {/each}

          </svg>
        </div>
      </div>

      <!-- Controls column -->
      <div class="controls-col">

        <!-- Upload / Camera buttons -->
        <input
          type="file"
          accept=".png,image/png"
          bind:this={fileInputEl}
          on:change={handleFileUpload}
          style="display:none"
        />
        <div class="upload-row">
          <button class="ctrl-btn upload-btn" on:click={() => fileInputEl.click()} disabled={uploading}>
            <img src={imageIcon} alt="" class="btn-icon" />
            <span>{uploading ? 'Processing…' : 'Upload Image'}</span>
          </button>
          <button class="ctrl-btn camera-btn" on:click={() => goto(`/controller-studio/pose-library/edit/capture?id=${poseId}&name=${encodeURIComponent(poseName)}`)}>
            <img src={cameraIcon} alt="" class="btn-icon" />
            <span>Pose for the Camera</span>
          </button>
        </div>

        <!-- Detection + Test Pose row -->
        <div class="detection-row">
          <!-- Detection Method card -->
          <div class="detection-card">
            <h3 class="detection-heading">Detection Method</h3>

            <div class="method-toggle">
              <button
                class="method-btn"
                class:method-active={detectionMethod === 'cosine'}
                on:click={() => { detectionMethod = 'cosine'; savePose(); }}
              >Cosine Similarity</button>
              <button
                class="method-btn"
                class:method-active={detectionMethod === 'relative'}
                on:click={() => { detectionMethod = 'relative'; savePose(); }}
              >Relative Positioning</button>
            </div>

            {#if detectionMethod === 'cosine'}
              <div class="similarity-section">
                <h3 class="detection-heading">Minimum Similarity</h3>
                <div class="slider-wrap">
                  <div class="slider-track">
                    <div class="slider-fill" style="width: {minSimilarity}%"></div>
                  </div>
                  <span class="slider-pct">{minSimilarity}%</span>
                  <input
                    type="range"
                    class="slider-input"
                    min="0"
                    max="100"
                    bind:value={minSimilarity}
                    on:input={onSliderChange}
                  />
                </div>
              </div>
            {/if}
          </div>

          <!-- Test Pose + Export/Delete column -->
          <div class="test-col">
            <button class="ctrl-btn test-btn" on:click={() => poseId && goto(`/controller-studio/pose-library/edit/test?id=${poseId}&name=${encodeURIComponent(poseName)}`)}>
              <img src={labFlaskIcon} alt="" class="test-icon" />
              <span>Test Pose</span>
            </button>

            <!-- Export Pose -->
            <!-- <button class="ctrl-btn export-btn">
              <span>Export Pose</span>
            </button> -->

            <!-- Delete Pose -->
            <button class="ctrl-btn delete-btn" on:click={async () => { if (poseId) { await invoke('delete_pose', { poseId }); goto('/controller-studio/pose-library'); } }}>
              <span>Delete Pose</span>
            </button>
          </div>
        </div>

      </div>
    </div>
  </div>

  <!-- ── Right sidebar ── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <h2 class="sidebar-title">Pose Reference Images</h2>
      <p class="sidebar-subtitle">Select images that best match your target pose. Unselect or delete any inaccurate ones.</p>
    </div>

    <div class="sidebar-scroll">
      {#each refImages as img (img.image_id)}
        <div class="ref-card">
          <img src="data:image/png;base64,{img.data_b64}" alt="Pose reference" class="ref-image" />
          <button class="ref-remove-btn" on:click={() => removeRef(img.image_id)} aria-label="Remove image">
            <img src={crossRedIcon} alt="Remove" class="ref-icon" />
          </button>
          <button class="ref-check-btn" on:click={() => toggleRef(img.image_id)} aria-label="Toggle selection">
            <img src={img.active ? checkedIcon : uncheckedIcon} alt={img.active ? 'Selected' : 'Unselected'} class="ref-icon" />
          </button>
        </div>
      {/each}
    </div>
  </aside>

</div>

<style>
  /* ── Root layout ── */
  .pose-editor {
    display: flex;
    height: 100vh;
    overflow: hidden;
    background-color: var(--color-tertiary-3);
    font-family: var(--font-primary);
  }

  /* ── Main panel ── */
  .main-panel {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: 3vh 3.5vw 4vh 3.5vw;
    display: flex;
    flex-direction: column;
  }

  /* ── Nav ── */
  .top-bar {
    padding-bottom: 3vh;
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

  /* ── Title ── */
  .title-row {
    margin-bottom: 2vh;
  }

  .title-wrap {
    display: inline-block;
    position: relative;
  }

  .pose-title {
    font-weight: var(--font-weight-Huge);
    font-size: var(--font-size-Huge);
    line-height: var(--line-height-Huge);
    font-style: var(--font-style-Huge);
    color: var(--color-dark-1);
    margin: 0;
  }

  .title-input {
    font-weight: var(--font-weight-Huge);
    font-size: var(--font-size-Huge);
    line-height: var(--line-height-Huge);
    font-style: var(--font-style-Huge);
    font-family: var(--font-primary);
    color: var(--color-dark-1);
    background: rgba(255, 255, 255, 0.5);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    outline: none;
    padding: 0.25rem 0.75rem;
    flex: 1;
    max-width: 60%;
  }

  /* ── Description ── */
  .desc-row {
    display: flex;
    align-items: flex-start;
    gap: 1.25rem;
    margin-bottom: 3vh;
    max-width: 70%;
  }

  .pose-desc {
    font-weight: var(--font-weight-H4);
    font-size: var(--font-size-H4);
    line-height: var(--line-height-H4);
    color: var(--color-dark-1);
    margin: 0;
    flex: 1;
  }

  .desc-input {
    font-weight: 700;
    font-size: var(--font-size-H5);
    line-height: 1.9rem;
    font-family: var(--font-primary);
    color: var(--color-dark-1);
    background: rgba(255, 255, 255, 0.5);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    outline: none;
    padding: 0.5rem 0.75rem;
    resize: vertical;
    min-height: 10vh;
    flex: 1;
    width: 100%;
  }

  /* ── Pencil buttons ── */
  .pencil-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    flex-shrink: 0;
    position: absolute;
    top: -0.5rem;
    right: -2.75rem;
  }

  .pencil-btn-desc {
    position: static;
    margin-top: 0.25rem;
    align-self: flex-start;
  }

  .pencil-icon {
    width: 2rem;
    height: 2rem;
    opacity: 0.85;
  }

  .pencil-btn:hover .pencil-icon {
    opacity: 1;
  }

  /* ── Editor area: skeleton + controls ── */
  .editor-area {
    display: flex;
    gap: 2.5vw;
    align-items: flex-start;
    flex: 1;
  }

  /* ── Skeleton viewport ── */
  .skeleton-viewport {
    width: 31vw;
    height: 54vh;
    background-color: var(--color-tertiary-1);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    overflow: hidden;
    cursor: grab;
    position: relative;
    flex-shrink: 0;
    user-select: none;
  }

  .skeleton-viewport.panning {
    cursor: grabbing;
  }

  .skeleton-inner {
    position: absolute;
    top: 50%;
    left: 50%;
    transform-origin: center center;
  }

  /* ── Controls column ── */
  .controls-col {
    display: flex;
    flex-direction: column;
    gap: 1.5vh;
    flex: 1;
    min-width: 0;
  }

  /* ── Shared ctrl-btn base ── */
  .ctrl-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H5);
    font-size: var(--font-size-H5);
    color: var(--color-dark-1);
    padding: 0.85rem 1.35rem;
    transition: background-color 0.1s, color 0.1s;
  }

  .ctrl-btn:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  .btn-icon {
    width: 2rem;
    height: 2rem;
    flex-shrink: 0;
  }

  /* Upload row */
  .upload-row {
    display: flex;
    gap: 1.5vw;
  }

  .upload-btn {
    background-color: var(--color-primary-2);
    flex: 1;
  }

  .camera-btn {
    background-color: var(--color-secondary-1);
    flex: 1.3;
  }

  /* Detection row */
  .detection-row {
    display: flex;
    gap: 1.5vw;
    align-items: flex-start;
  }

  .detection-card {
    background-color: var(--color-secondary-2);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    padding: 1.25rem 1.5rem;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .detection-heading {
    font-weight: var(--font-weight-H4);
    font-size: var(--font-size-H4);
    line-height: var(--line-height-H4);
    color: var(--color-dark-1);
    margin: 0;
  }

  /* Method toggle pill */
  .method-toggle {
    display: flex;
    background-color: var(--color-secondary-3);
    border-radius: 999px;
    padding: 0.3rem;
    gap: 0;
    border: var(--stroke-width-xs) solid var(--color-dark-1);
  }

  .method-btn {
    flex: 1;
    background: transparent;
    border: none;
    border-radius: 999px;
    color: var(--color-white);
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H6);
    font-size: var(--font-size-H6);
    line-height: var(--line-height-H6);
    padding: 0.45rem 0.85rem;
    cursor: pointer;
    transition: background-color 0.15s, color 0.15s;
    white-space: nowrap;
  }

  .method-btn.method-active {
    background-color: var(--color-primary-1);
    color: var(--color-dark-1);
  }

  /* Similarity slider */
  .similarity-section {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .slider-wrap {
    position: relative;
    height: 5vh;
  }

  .slider-track {
    position: absolute;
    inset: 0;
    border-radius: 999px;
    border: var(--stroke-width-s) solid var(--color-dark-1);
    overflow: hidden;
    pointer-events: none;
    background-color: var(--color-background);
  }

  .slider-fill {
    height: 100%;
    background-color: var(--color-primary-4);
    transition: width 0.05s;
    pointer-events: none;
    border-radius: 4rem;
  }

  .slider-pct {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
    font-family: var(--font-primary);
    font-weight: 700;
    font-size: 1.125rem;
    color: var(--color-dark-1);
    z-index: 2;
  }

  .slider-input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
    margin: 0;
    z-index: 3;
  }

  .test-col {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    width: 8vw;
    flex-shrink: 0;
  }

  /* Test Pose button */
  .test-btn {
    background-color: var(--color-primary-3);
    flex-direction: column;
    padding: 1.25rem 1.5rem;
    align-self: stretch;
    min-width: 0;
    box-shadow: var(--shadow-m);
    gap: 0.5rem;
  }

  .test-icon {
    width: 3rem;
    height: 3rem;
    flex-shrink: 0;
    filter: invert(0);
    transition: filter 0.1s;
  }

  .test-btn:hover .test-icon {
    filter: invert(1);
  }

  /* Export Pose */
  .export-btn {
    background-color: var(--color-tertiary-2);
  }

  /* Delete Pose */
  .delete-btn {
    background-color: var(--color-window);
  }

  /* ── Right sidebar ── */
  .sidebar {
    width: 28vw;
    min-width: 0;
    flex-shrink: 0;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-left: var(--stroke-width-s) solid var(--color-dark-1);
  }

  .sidebar-header {
    background-color: var(--color-secondary-4);
    padding: 2vh 2vw 1.75vh 2vw;
    border-bottom: var(--stroke-width-s) solid var(--color-dark-1);
    flex-shrink: 0;
  }

  .sidebar-title {
    font-weight: var(--font-weight-H3);
    font-size: var(--font-size-H3);
    line-height: var(--line-height-H3);
    color: var(--color-dark-1);
    margin: 0 0 1vh 0;
  }

  .sidebar-subtitle {
    font-size: var(--font-size-Body-L);
    line-height: var(--line-height-Body-L);
    color: var(--color-dark-1);
    margin: 0;
  }

  .sidebar-scroll {
    flex: 1;
    overflow-y: auto;
    background-color: var(--color-primary-3);
    padding: 2vh 2vw;
    display: flex;
    flex-direction: column;
    gap: 2vh;
  }

  /* ── Reference image cards ── */
  .ref-card {
    background-color: var(--color-background);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-s);
    position: relative;
    overflow: hidden;
    flex-shrink: 0;
    height: 26vh;
  }

  .ref-image {
    width: 100%;
    height: 100%;
    display: block;
    min-width: 0;
    min-height: 0;
    object-fit: cover;
  }

  .ref-remove-btn,
  .ref-check-btn {
    position: absolute;
    top: 0.5rem;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .ref-remove-btn { left: 0.5rem; }
  .ref-check-btn  { right: 0.5rem; }

  .ref-remove-btn:hover,
  .ref-check-btn:hover {
    opacity: 0.75;
  }

  .ref-icon {
    width: 2rem;
    height: 2rem;
  }
</style>
