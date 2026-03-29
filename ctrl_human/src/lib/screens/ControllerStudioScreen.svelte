<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import '$lib/styles/tokens.css';
  import frogImg from '$lib/assets/images/sitting_toad_posing_webcam.png';
  import checkedIcon from '$lib/assets/icons/checked.svg';
  import uncheckedIcon from '$lib/assets/icons/unchecked.svg';

  function goHome() { goto('/'); }
  function goToPoseLibrary() { goto('/controller-studio/pose-library'); }
  function goToControllerLibrary() { goto('/controller-studio/controller-library'); }

  type Scheme = { controller_id: string; name: string; hotkey: boolean };
  let schemes: Scheme[] = [];

  onMount(async () => {
    try {
      const controllers = await invoke<{ controller_id: string; title: string; hotkey: boolean }[]>('load_controllers');
      schemes = controllers.map(c => ({ controller_id: c.controller_id, name: c.title, hotkey: c.hotkey }));
    } catch (e) { console.error('load_controllers failed:', e); }
  });

  async function toggleScheme(index: number) {
    const scheme = schemes[index];
    const newHotkey = !scheme.hotkey;
    try {
      await invoke('set_controller_hotkey', { controllerId: scheme.controller_id, hotkey: newHotkey });
      schemes[index] = { ...scheme, hotkey: newHotkey };
      schemes = schemes;
    } catch (e) { console.error('set_controller_hotkey failed:', e); }
  }
</script>

<main class="studio">
  <div class="content-area">
    <nav class="top-bar">
      <button class="back-btn" on:click={goHome}>
        <svg class="back-arrow" viewBox="0 0 33 40" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M11,5 L21,5 L21,19 L28,19 L26.083,23.923 L24.167,23.923 L20.333,31.308 L18.554,31.308 L17.778,35 L15.222,35 L14.583,31.308 L12.667,31.308 L9.472,23.923 L6.917,23.923 L5,19 L11,19 Z" />
        </svg>
        <span class="back-text">Home</span>
      </button>
    </nav>

    <div class="cards-area">
      <div class="card pose-card">
        <h2 class="card-heading">Pose Maker</h2>
        <p class="card-text">Pose in front of the camera, then tweak the captured pose landmarks to build your own custom pose template.</p>
        <button class="cta cta-pose" on:click={goToPoseLibrary}>Selfie Time</button>
      </div>

      <div class="webcam-frame">
        <img src={frogImg} alt="Frog posing at webcam" class="webcam-img" />
      </div>

      <div class="card builder-card">
        <h2 class="card-heading">Controller Builder</h2>
        <p class="card-text">Use your pose templates to turn your body movements into a fully functional game controller.</p>
        <button class="cta cta-builder" on:click={goToControllerLibrary}>Yeehaw</button>
      </div>

      <h1 class="studio-title">Controller<br>Studio</h1>
    </div>
  </div>

  <aside class="sidebar">
    <div class="sidebar-header">
      <h2 class="sidebar-heading">Controller Schemes</h2>
      <p class="sidebar-desc">Choose which control schemes are enabled or hidden from the dropdown selector</p>
    </div>
    <div class="sidebar-list">
      {#each schemes as scheme, i}
        <button class="scheme-item" on:click={() => toggleScheme(i)}>
          <span class="scheme-name">{scheme.name}</span>
          <img src={scheme.hotkey ? checkedIcon : uncheckedIcon} alt={scheme.hotkey ? 'Enabled' : 'Disabled'} class="scheme-checkbox" />
        </button>
      {/each}
    </div>
  </aside>
</main>

<style>
  /* ── Layout ── */
  .studio {
    width: 100%;
    height: 100vh;
    display: flex;
    background-color: var(--color-tertiary-1);
    box-sizing: border-box;
    overflow: hidden;
  }

  .content-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  /* ── Nav ── */
  .top-bar {
    padding: 4rem 4rem;
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

  /* ── Cards area ── */
  .cards-area {
    position: relative;
    flex: 1;
    margin: 0 4rem;
  }

  /* ── Shared card styles ── */
  .card {
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-l);
    padding-top: 4rem;
    padding-bottom: 4rem;
    padding-left: 2rem;
    padding-right: 2rem;
    box-sizing: border-box;
  }

  .card-heading {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H1);
    font-size: var(--font-size-H1);
    line-height: var(--line-height-H1);
    color: var(--color-dark-1);
    margin: 0 0 3rem 0;
  }

  .card-heading-light {
    color: var(--color-white);
  }

  .card-text {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H4);
    font-size: var(--font-size-H4);
    line-height: var(--line-height-H4);
    color: var(--color-dark-1);
    margin: 0 0 3rem 0;
  }

  .card-text-light {
    color: var(--color-white);
  }

  .cta {
    border: var(--stroke-width-s) solid var(--color-dark-1);
    border-radius: 0;
    padding: 1.5rem 1.5rem;
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H3);
    font-size: var(--font-size-H3);
    line-height: var(--line-height-H3);
    color: var(--color-dark-1);
    box-shadow: var(--shadow-m);
  }

  .cta:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  /* ── Pose Maker card ── */
  .pose-card {
    position: absolute;
    top: 0;
    left: 0;
    width: 37.5rem;
    background-color: var(--color-primary-2);
  }

  .cta-pose {
    background-color: var(--color-secondary-1);
  }

  /* ── Webcam frame ── */
  .webcam-frame {
    position: absolute;
    top: -7.865rem;
    left: 41.475rem;
    width: 22.0625rem;
    border: var(--stroke-width-s) solid var(--color-dark-1);
    background-color: var(--color-dark-1);
  }

  .webcam-titlebar {
    background-color: #444;
    padding: 0.4rem 0.75rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .rec-dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background-color: #ff3b3b;
    display: inline-block;
  }

  .rec-label {
    font-family: var(--font-primary);
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--color-white);
  }

  .titlebar-icons {
    display: flex;
    gap: 0.4rem;
    font-size: 0.65rem;
  }

  .tb-icon {
    color: var(--color-white);
    opacity: 0.7;
  }

  .titlebar-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .progress-bar {
    display: flex;
    width: 4rem;
    height: 0.45rem;
    border-radius: 0.25rem;
    overflow: hidden;
  }

  .progress-red {
    flex: 1;
    background-color: #ff3b3b;
  }

  .progress-green {
    flex: 1;
    background-color: #30b57f;
  }

  .window-dots {
    display: flex;
    gap: 0.3rem;
  }

  .wdot {
    width: 0.6rem;
    height: 0.6rem;
    border-radius: 50%;
    display: inline-block;
  }

  .wdot-red { background-color: #ff5f57; }
  .wdot-yellow { background-color: #febc2e; }
  .wdot-green { background-color: #28c840; }

  .webcam-img {
    width: 100%;
    display: block;
  }

  /* ── Controller Builder card ── */
  .builder-card {
    position: absolute;
    top: 18.18rem;
    left: 41.4375rem;
    width: 37.5rem;
    background-color: var(--color-window);
  }

  .cta-builder {
    background-color: var(--color-secondary-2);
  }

  /* ── Studio title ── */
  .studio-title {
    position: absolute;
    bottom: 3.88125rem;
    left: 0;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-Huge);
    font-size: var(--font-size-Huge);
    line-height: var(--line-height-Huge);
    font-style: var(--font-style-Huge);
    color: var(--color-dark-1);
    margin: 0;
    text-align: right;
  }

  /* ── Sidebar ── */
  .sidebar {
    width: 33.0625rem;
    display: flex;
    flex-direction: column;
    height: 100vh;
    flex-shrink: 0;
  }

  .sidebar-header {
    background-color: var(--color-secondary-4);
    padding: 2rem 2rem;
    border-left: var(--stroke-width-s) solid var(--color-dark-1);
    border-bottom: var(--stroke-width-s) solid var(--color-dark-1);
  }

  .sidebar-heading {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H2);
    font-size: var(--font-size-H2);
    line-height: var(--line-height-H2);
    color: var(--color-dark-1);
    margin: 0 0 1rem 0;
  }

  .sidebar-desc {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-Body-M);
    font-size: var(--font-size-Body-M);
    line-height: var(--line-height-Body-M);
    color: var(--color-dark-1);
    margin: 0;
  }

  .sidebar-list {
    background-color: var(--color-primary-3);
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 2rem;
    border-left: var(--stroke-width-s) solid var(--color-dark-1);
  }

  .scheme-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2rem 2rem;
    background-color: var(--color-white);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    border-radius: 0;
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H3);
    font-size: var(--font-size-H3);
    line-height: var(--line-height-H3);
    color: var(--color-dark-1);
    width: 100%;
    box-sizing: border-box;
  }

  .scheme-item:hover {
    background-color: var(--color-grey-2);
  }

  .scheme-checkbox {
    width: 1.75rem;
    height: 1.75rem;
  }
</style>
