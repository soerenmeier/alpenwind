<script lang="ts">
	import { onDestroy, getContext } from 'svelte';
	import DateTime from 'fire/time/DateTime';
	import { apps } from '../lib/apps';
	import { getCore } from 'core-lib';

	const cl = getCore();
	const { user } = cl;

	let dayStr = '';
	let dateStr = '';
	let timeStr = '';
	function updateTimeStr() {
		const now = new DateTime();
		dayStr = now.toStrDay();
		dateStr = `${now.date}. ${now.toStrMonth()} ${now.year}`;
		timeStr = now.toStrShortTime();
	}

	updateTimeStr();
	const timeInterval = setInterval(updateTimeStr, 5000);

	onDestroy(() => {
		clearInterval(timeInterval);
	});
</script>

<div id="apps" class="abs-full bg-image">
	<div class="welcome">
		<h1>Hallo {$user.name}!</h1>
		<p class="date-style date-cont">
			<span class="day">{dayStr}</span>
			<span class="date">{dateStr}</span>
			<span class="time">{timeStr}</span>
		</p>
	</div>

	<div class="apps">
		{#each apps as app}
			<a href={app.uri()} class="app">
				<span class="icon" style="--icon: url('{app.icon()}')"></span>
				<h4>{app.name()}</h4>
			</a>
		{/each}
	</div>
</div>

<style>
	#apps {
		padding: 120px;
	}

	.welcome {
		display: flex;
		margin-bottom: 70px;
		padding: 25px;
		border: 1px solid var(--blur-border-color);
		backdrop-filter: blur(20px) brightness(0.6);
		border-radius: 10px;
		justify-content: space-between;
		align-items: center;
	}

	h1 {
		font-size: 25px;
	}

	.date-style {
		font-size: 20px;
		font-weight: 500;
		color: rgba(255, 255, 255, 0.69);
	}

	.day::after,
	.date::after {
		content: ',';
	}

	.apps {
		display: grid;
		grid-template-columns: repeat(6, 1fr);
		grid-gap: calc((100% - 600px) / 5);
	}

	.app {
		display: block;
		text-decoration: none;
		overflow: hidden;
	}

	.icon {
		display: block;
		padding-top: 100%;
		border-radius: 25%;
		background-color: #fff;
		background-image: var(--icon);
		background-size: 70%;
		background-repeat: no-repeat;
		background-position: center;
	}

	h4 {
		margin-top: 8px;
		text-align: center;
	}

	@media (max-width: 1300px) {
		#apps {
			padding: 60px;
		}

		.apps {
			grid-template-columns: repeat(5, 1fr);
			grid-gap: calc((100% - 450px) / 4);
		}
	}

	@media (max-width: 900px) {
		#apps {
			padding: 30px;
		}

		.welcome {
			margin-bottom: 40px;
		}

		h1 {
			font-size: 20px;
		}

		.date-style {
			font-size: 18px;
		}
	}

	@media (max-width: 500px) {
		#apps {
			padding: 20px;
		}

		.day {
			display: none;
		}

		.date {
			display: none;
		}

		.apps {
			grid-template-columns: repeat(4, 1fr);
			grid-gap: calc((100% - 300px) / 3);
		}

		h4 {
			font-size: 13px;
		}
	}
</style>
