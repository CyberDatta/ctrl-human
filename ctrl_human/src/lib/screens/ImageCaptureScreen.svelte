<script lang="ts">
  import { goto } from '$app/navigation';
  import '$lib/styles/tokens.css';
  import cameraIcon from '$lib/assets/icons/camera.svg';
  import crossRedIcon from '$lib/assets/icons/cross.svg';
  import switchCardIcon from '$lib/assets/icons/switch_card.svg';
  import ecstaticIcon from '$lib/assets/icons/ecstatic.svg';

  let poseName = 'Untitled Pose 7';

  // Captured images (placeholder cards)
  let captures: { id: number }[] = [
    { id: 1 },
    { id: 2 },
    { id: 3 },
    { id: 4 },
    { id: 5 },
    { id: 6 },
    { id: 7 },
    { id: 8 },
    { id: 9 },
  ];

  function removeCapture(id: number) {
    captures = captures.filter(c => c.id !== id);
  }

  function goBack() {
    goto('/controller-studio/pose-library/edit');
  }
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

    <!-- Webcam preview -->
    <div class="video-preview"></div>

    <!-- Action buttons -->
    <div class="action-col">
      <button class="action-btn selfie-btn">
        <img src={cameraIcon} alt="" class="action-icon" />
        <span>Selfie Time</span>
      </button>

      <button class="action-btn switch-btn">
        <img src={switchCardIcon} alt="" class="action-icon" />
        <span>Switch Camera</span>
      </button>

      <button class="action-btn over-btn">
        <img src={ecstaticIcon} alt="" class="action-icon" />
        <span>Selfie Time Over</span>
      </button>
    </div>
  </div>

  <!-- Horizontally scrollable captures strip -->
  <div class="captures-strip">
    {#each captures as cap (cap.id)}
      <div class="capture-card">
        <button class="remove-btn" on:click={() => removeCapture(cap.id)} aria-label="Remove capture">
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
    overflow: hidden;
    background-color: var(--color-primary-3);
    font-family: var(--font-primary);
    padding: 2.5rem 3rem 0 3rem;
    box-sizing: border-box;
  }

  /* ── Nav ── */
  .top-bar {
    flex-shrink: 0;
    margin-bottom: 4rem;
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
    display: flex;
    gap: 2rem;
    align-items: center;
    justify-content: center;
  }

  /* ── Webcam preview ── */
  .video-preview {
    /* flex: 1; */
    width: 56.25rem;
    height: 33.875rem;
    background-color: var(--color-background);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
  }

  /* ── Action buttons column ── */
  .action-col {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    width: 19rem;
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
    width: 26.9375rem;
    height: 6.5;
  }

  .action-btn:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  .selfie-btn {
    background-color: var(--color-secondary-2);
  }

  .switch-btn {
    background-color: var(--color-primary-2);
  }

  .over-btn {
    background-color: var(--color-secondary-4);
  }

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
    gap: 1.25rem;
    overflow-x: auto;
    padding: 1.5rem 3rem;
    margin: 4rem -3rem 0;
    scrollbar-width: thin;
    scrollbar-color: var(--color-dark-1) transparent;
  }

  .captures-strip::-webkit-scrollbar {
    height: 0.4rem;
  }

  .captures-strip::-webkit-scrollbar-thumb {
    background-color: var(--color-dark-1);
    border-radius: 999px;
  }

  .capture-card {
    flex-shrink: 0;
    width: 22.3125rem;
    height: 13.5rem;
    background-color: var(--color-background);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    position: relative;
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

  .remove-btn:hover {
    opacity: 0.75;
  }

  .remove-icon {
    width: 2rem;
    height: 2rem;
  }
</style>
