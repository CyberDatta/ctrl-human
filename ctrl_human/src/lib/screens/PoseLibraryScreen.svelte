<script lang="ts">
  import { goto } from '$app/navigation';
  import '$lib/styles/tokens.css';
  import magnifyingGlass from '$lib/assets/icons/magnifying_glass.svg';
  import cameraIcon from '$lib/assets/icons/camera.svg';
  import imageIcon from '$lib/assets/icons/image.svg';
  import uploadIcon from '$lib/assets/icons/arrow_for_upload.svg';
  import pencilIcon from '$lib/assets/icons/pencil.svg';
  import downloadIcon from '$lib/assets/icons/arrow_for_download.svg';
  import trashIcon from '$lib/assets/icons/trash_can.svg';

  function goBack() {
    goto('/controller-studio');
  }

  let searchQuery = '';

  // Placeholder pose data
  let poses = [
    { name: 'Wave with both Hands', image: '' },
    { name: 'Wave with one Hand', image: '' },
  ];

  $: filteredPoses = poses.filter(pose =>
    pose.name.toLowerCase().includes(searchQuery.toLowerCase())
  );
</script>

<main class="pose-library">
  <nav class="top-bar">
    <button class="back-btn" on:click={goBack}>
      <svg class="back-arrow" viewBox="0 0 33 40" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M11,5 L21,5 L21,19 L28,19 L26.083,23.923 L24.167,23.923 L20.333,31.308 L18.554,31.308 L17.778,35 L15.222,35 L14.583,31.308 L12.667,31.308 L9.472,23.923 L6.917,23.923 L5,19 L11,19 Z" />
      </svg>
      <span class="back-text">Controller Studio</span>
    </button>
  </nav>

  <h1 class="page-title">Pose Library</h1>

  <div class="toolbar">
    <div class="search-bar">
      <img src={magnifyingGlass} alt="Search" class="search-icon" />
      <input
        type="text"
        class="search-input"
        placeholder="Search with name..."
        bind:value={searchQuery}
      />
    </div>

    <div class="action-buttons">
      <button class="action-btn action-btn-webcam">
        <img src={cameraIcon} alt="Camera" class="action-icon" />
        <span>Create With Webcam</span>
      </button>
      <button class="action-btn action-btn-upload">
        <img src={imageIcon} alt="Image" class="action-icon" />
        <span>Upload Image</span>
      </button>
      <button class="action-btn action-btn-import">
        <img src={uploadIcon} alt="Import" class="action-icon" />
        <span>Import From Pose File</span>
      </button>
    </div>
  </div>

  <div class="poses-grid">
    {#each filteredPoses as pose}
      <div class="pose-card">
        <div class="pose-card-icons">
          <button class="icon-btn">
            <img src={pencilIcon} alt="Edit" class="card-icon" />
          </button>
          <button class="icon-btn">
            <img src={downloadIcon} alt="Download" class="card-icon" />
          </button>
          <button class="icon-btn">
            <img src={trashIcon} alt="Delete" class="card-icon" />
          </button>
        </div>
        <div class="pose-card-image">
          {#if pose.image}
            <img src={pose.image} alt={pose.name} />
          {/if}
        </div>
        <div class="pose-card-label">
          <span class="pose-name">{pose.name}</span>
        </div>
      </div>
    {/each}
  </div>
</main>

<style>
  .pose-library {
    width: 100%;
    height: 100vh;
    background-color: var(--color-primary-1);
    box-sizing: border-box;
    overflow-y: auto;
    padding: 0 4rem 4rem 4rem;
  }

  /* ── Nav ── */
  .top-bar {
    padding: 4rem 0 2rem 0;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 1rem;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .back-arrow {
    height: 4.6875rem;
    transform: rotate(90deg);
  }

  .back-arrow path {
    fill: var(--color-background);
    stroke: var(--color-primary-outline);
    stroke-width: var(--stroke-width-xs);
    vector-effect: non-scaling-stroke;
  }

  .back-text {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H1);
    font-size: var(--font-size-H1);
    line-height: var(--line-height-H1);
    color: var(--color-background);
    -webkit-text-stroke: var(--stroke-width-xs) var(--color-primary-outline);
  }

  /* ── Title ── */
  .page-title {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-Huge);
    font-size: var(--font-size-Huge);
    line-height: var(--line-height-Huge);
    font-style: var(--font-style-Huge);
    color: var(--color-dark-1);
    margin: 0 0 2rem 0;
  }

  /* ── Toolbar ── */
  .toolbar {
    display: flex;
    align-items: center;
    gap: 2rem;
    margin-bottom: 2rem;
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background-color: var(--color-background);
    border: var(--stroke-width-s) solid var(--color-secondary-4);
    padding: 0.75rem 1rem;
    flex: 0 1 32rem;
  }

  .search-icon {
    width: 1.5rem;
    height: 1.5rem;
    flex-shrink: 0;
  }

  .search-input {
    border: none;
    background: none;
    outline: none;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-Body-M);
    font-size: var(--font-size-Body-M);
    line-height: var(--line-height-Body-M);
    color: var(--color-dark-1);
    width: 100%;
  }

  .search-input::placeholder {
    color: var(--color-dark-2);
  }

  .action-buttons {
    display: flex;
    gap: 1.5rem;
    margin-left: auto;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1.5rem;
    border: var(--stroke-width-s) solid var(--color-dark-1);
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H5);
    font-size: var(--font-size-H5);
    line-height: var(--line-height-H5);
    color: var(--color-dark-1);
    box-shadow: var(--shadow-m);
  }

  .action-btn:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  .action-btn-webcam {
    background-color: var(--color-secondary-2);
  }

  .action-btn-upload {
    background-color: var(--color-primary-2);
  }

  .action-btn-import {
    background-color: var(--color-secondary-1);
  }

  .action-icon {
    width: 2rem;
    height: 2rem;
  }

  /* ── Poses grid ── */
  .poses-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 2rem;
  }

  .pose-card {
    width: 16rem;
    border: var(--stroke-width-s) solid var(--color-dark-1);
    background-color: var(--color-primary-3);
    box-shadow: var(--shadow-m);
    display: flex;
    flex-direction: column;
  }

  .pose-card-icons {
    display: flex;
    justify-content: flex-end;
    gap: 0.25rem;
    padding: 0.5rem 0.5rem 0 0.5rem;
  }

  .icon-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-btn:hover {
    opacity: 0.7;
  }

  .card-icon {
    width: 2rem;
    height: 2rem;
  }

  .pose-card-image {
    background-color: var(--color-background);
    margin: 0.5rem;
    height: 12rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .pose-card-image img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .pose-card-label {
    padding: 0.75rem 1rem;
  }

  .pose-name {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H5);
    font-size: var(--font-size-H5);
    line-height: var(--line-height-H5);
    color: var(--color-white);
  }
</style>
