<script lang="ts">
  import '$lib/styles/tokens.css';
</script>

<main class="home">
  <h1>Ctrl+Human</h1>

  <div class="row">
    <div class="box box-left"></div>
    <div class="box box-right"></div>
  </div>
</main>

<style>
  .home {
    width: 100%;
    height: 100vh;
    padding: 2rem;
    box-sizing: border-box;
    background-color: var(--color-secondary-2);
  }

  h1 {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-Huge);
    font-size: var(--font-size-Huge);
    line-height: var(--line-height-Huge);
    font-style: var(--font-style-Huge);
    color: var(--color-dark-1);
    margin: 0;
  }

  .row {
    display: flex;
    gap: 1.5rem;
    margin-top: 1.5rem;
    flex: 1;
  }

  .box {
    flex: 1;
    border-radius: 1rem;
  }

  .box-left {
    background-color: var(--color-background);
  }

  .box-right {
    background-color: var(--color-secondary-3);
  }
</style>
