<script lang="ts">
  export let disabled = false;
  export let type: "button" | "submit" | "reset";
</script>

<button type={type} disabled={disabled}>
	<slot />
</button>

<style>
	button {
		background-color: var(--brand-primary);
		color: #fff;
		border: 0;
		font-family: var(--font-main);
		border-radius: 4px;
		width: 100%;
		height: 48px;
		font-size: 14px;
		line-height: 1;
		cursor: pointer;
		transition: background-color 0.1s;
	}
  
  button:focus {
    outline: var(--brand-primary) solid 2px;
    outline-offset: 2px;
  }

  button:hover {
          background-color: var(--brand-primary-dark);
        }

        button:disabled {
          opacity: 0.1;
          cursor: default;
        }
</style>
