<script lang="ts">
	import {
		Button,
		Card,
		CardBody,
		Col,
		Container,
		FormGroup,
		Input,
		Label,
		Row
	} from 'sveltestrap';

	import { DefaultApi } from '$lib/openapi';
	const api = new DefaultApi();

	interface Task {
		id: number;
		title: string;
		done: boolean;
	}

	let tasks: Task[] = [];

	let addTaskTitle = '';

	async function getTaskList() {
		return await api.tasksGet();
	}

	async function addTask() {
        const body = {title: addTaskTitle};
        addTaskTitle = "";

		await api.tasksPost({
			postTasks: body
		})

		tasks = await getTaskList();
	}

	async function removeTask(id) {
		await api.tasksIdDelete({
			id
		});

        tasks = await getTaskList();
	}

    async function toggleTask(task: Task) {
		await api.tasksIdPatch({
			id: task.id,
			patchTask: {
				done: task.done
			}
		});

        tasks = await getTaskList();
    }

	getTaskList()
		.then((res) => (tasks = res))
		.catch(console.error);
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<Container>
	<FormGroup>
		<Label for="add-task-input">タスクの追加</Label>
		<Row>
			<Col xs="10">
				<Input
					id="add-task-input"
					bind:value={addTaskTitle}
					on:keypress={(event) => {
						if (event.key === 'Enter') {
							addTask();
						}
					}}
				/>
			</Col>
			<Col xs="2">
				<Button on:click={addTask} disabled={addTaskTitle ? '' : 'disabled'}>追加</Button>
			</Col>
		</Row>
	</FormGroup>

	{#each tasks as task (task.id)}
		<Card>
			<CardBody>
				<div class="task">
					<div class="task-title">
						<FormGroup style="margin: 0 !important;">
							<input
								id={`task-checked-${task.id}`}
								type="checkbox"
								class="form-check-input"
								bind:checked={task.done}
                                on:change={async () => toggleTask(task)}
							/>
							{' '}
							<Label class="fs-4" for={`task-checked-${task.id}`}>
								{#if task.done}
									<s>{task.title}</s>
								{:else}
									{task.title}
								{/if}
							</Label>
						</FormGroup>
					</div>
					<div class="close-button">
                        <button type="button" class="btn-close" on:click={async () => await removeTask(task.id)}></button>
					</div>
				</div>
			</CardBody>
		</Card>
	{/each}
</Container>

<style>
	.task-title {
		width: 99%;
	}

	.task {
		display: flex;
	}

	.close-button {
		display: flex;
		justify-content: flex-end;
	}

	.close-button > button {
		width: 2rem;
		height: 2rem;
	}

	.form-check-input {
		width: 32px;
		height: 32px;
		margin-right: 0.5rem;
	}

	h1 {
		width: 100%;
	}
</style>
