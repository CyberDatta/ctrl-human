<script lang="ts">
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import '$lib/styles/tokens.css';
  import magnifyingGlass from '$lib/assets/icons/magnifying_glass.svg';
  import uploadIcon from '$lib/assets/icons/arrow_for_upload.svg';
  import cameraIcon from '$lib/assets/icons/camera.svg';
  import pencilIcon from '$lib/assets/icons/pencil.svg';
  import downloadIcon from '$lib/assets/icons/arrow_for_download.svg';
  import trashIcon from '$lib/assets/icons/trash_can.svg';

  function goBack() {
    goto('/controller-studio');
  }

  let searchQuery = '';

  interface ControllerSummary {
    controller_id: string;
    title: string;
  }

  let controllers: ControllerSummary[] = [];

  onMount(async () => {
    controllers = await invoke<ControllerSummary[]>('load_controllers');
  });

  async function deleteController(controller_id: string) {
    try {
      await invoke('delete_controller', { controllerId: controller_id });
      controllers = await invoke<ControllerSummary[]>('load_controllers');
    } catch (e) {
      console.error('delete_controller failed:', e);
    }
  }

  async function createController() {
    const controller_id = await invoke<string>('create_controller');
    controllers = await invoke<ControllerSummary[]>('load_controllers');
    goto(`/controller-studio/controller-library/edit?id=${controller_id}`);
  }

  $: filteredControllers = controllers.filter(c =>
    c.title.toLowerCase().includes(searchQuery.toLowerCase())
  );
</script>

<main class="controller-library">
  <div class="page-content">
  <nav class="top-bar">
    <button class="back-btn" on:click={goBack}>
      <svg class="back-arrow" viewBox="0 0 33 40" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M11,5 L21,5 L21,19 L28,19 L26.083,23.923 L24.167,23.923 L20.333,31.308 L18.554,31.308 L17.778,35 L15.222,35 L14.583,31.308 L12.667,31.308 L9.472,23.923 L6.917,23.923 L5,19 L11,19 Z" />
      </svg>
      <span class="back-text">Controller Studio</span>
    </button>
  </nav>

  <h1 class="page-title">Controller Library</h1>

  <div class="toolbar">
    <div class="search-bar">
      <img src={magnifyingGlass} alt="Search" class="search-icon" />
      <div class="search-inner">
        <input
          type="text"
          class="search-input"
          placeholder="Search with name..."
          bind:value={searchQuery}
        />
      </div>
    </div>

    <div class="action-buttons">
      <!-- <button class="action-btn action-btn-import">
        <img src={uploadIcon} alt="Import" class="action-icon" />
        <span>Import From Controller File</span>
      </button> -->
      <button class="action-btn action-btn-create" on:click={createController}>
        <img src={cameraIcon} alt="Create" class="action-icon" />
        <span>Create New Controller Scheme</span>
      </button>
    </div>
  </div>

  <div class="controllers-grid">
    {#each filteredControllers as controller} 
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div class="controller-card" on:click={() => goto(`/controller-studio/controller-library/edit?id=${controller.controller_id}`)} style="cursor: pointer;">
        <div class="controller-card-icons">
          <button class="icon-btn" on:click|stopPropagation={() => deleteController(controller.controller_id)}>
            <img src={trashIcon} alt="Delete" class="card-icon" />
          </button>
        </div>
        <div class="controller-card-image"></div>
        <div class="controller-card-label">
          <span class="controller-name">{controller.title}</span>
        </div>
      </div>
    {/each}
  </div>
  </div>
</main>

<style>
  .controller-library {
    width: 100%;
    height: 100vh;
    background-color: var(--color-primary-1);
    box-sizing: border-box;
    overflow-y: auto;
  }

  .page-content {
    padding: 4vh 3.5vw;
    box-sizing: border-box;
  }

  /* ── Nav ── */
  .top-bar {
    padding: 0 0 4vh 0;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 2.25rem;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .back-arrow {
    height: 7vh;
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
    margin: 0 0 2vh 0;
  }

  /* ── Toolbar ── */
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0;
    margin-bottom: 3vh;
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    background-color: var(--color-secondary-4);
    padding: 1rem 2rem;
    flex: 0 1 33vw;
    box-shadow: var(--shadow-m);
    border: var(--stroke-width-s) solid var(--color-dark-1);
  }

  .search-icon {
    width: 3rem;
    height: 3rem;
    flex-shrink: 0;
  }

  .search-inner {
    border: var(--stroke-width-xs) solid var(--color-dark-1);
    background-color: var(--color-background);
    border-radius: var(--border-radius-s);
    flex: 1;
    padding: 0.4rem 0.75rem;
  }

  .search-input {
    border: none;
    background: none;
    outline: none;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H5);
    font-size: var(--font-size-H5);
    line-height: var(--line-height-H5);
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

  .action-btn-import {
    background-color: var(--color-secondary-1);
  }

  .action-btn-create {
    background-color: var(--color-secondary-2);
  }

  .action-icon {
    width: 2rem;
    height: 2rem;
  }

  /* ── Controllers grid ── */
  .controllers-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 2vw;
  }

  .controller-card {
    width: 17vw;
    height: 28vh;
    box-sizing: border-box;
    border: var(--stroke-width-s) solid var(--color-dark-1);
    background-color: var(--color-background);
    box-shadow: var(--shadow-m);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .controller-card-icons {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    padding: 1rem 1rem 0rem 0rem;
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

  .controller-card-image {
    flex: 1;
    min-height: 0;
  }

  .controller-card-label {
    background-color: var(--color-primary-3);
    padding: 0.75rem 1rem;
    border-top: var(--stroke-width-s) solid var(--color-dark-1);
  }

  .controller-name {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H5);
    font-size: var(--font-size-H5);
    line-height: var(--line-height-H5);
    color: var(--color-dark-1);
  }
</style>
