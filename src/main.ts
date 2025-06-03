import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
let application: HTMLElement | null;
declare global {
	interface Window {
		invoke: any;
		debounce: (
			callback: (...args: unknown[]) => unknown,
			wait: number,
		) => (...args: unknown[]) => void;
	}
}
window.invoke = invoke;
window.debounce = (callback: (...args: unknown[]) => unknown, wait: number) => {
	let timeoutId: undefined | number = undefined;
	return (...args: unknown[]) => {
		window.clearTimeout(timeoutId);
		timeoutId = window.setTimeout(() => callback(...args), wait);
	};
};

type ReplaceDirector = {
	target?: string;
	content: string;
	outerHTML?: boolean;
};

listen<{ target?: string; content: string }>("page_update", async (event) => {
	replaceContent(event.payload);
}).then((unlisten) => {
	window.addEventListener("unload", () => unlisten());
});

window.addEventListener("DOMContentLoaded", () => {
	application = document.querySelector("#application");
	if (application == null) {
		document.firstElementChild!.innerHTML = "<h1>Oh no</h1>";
		throw new Error("Invalid state :: Application not found");
	}
	invoke<ReplaceDirector>("page_x_current")
		.then(replaceContent)
		.catch(replaceError);
});

function replaceContent({ target, content, outerHTML }: ReplaceDirector): void {
	try {
		console.log(target, content);
		// no target returned. Don't need to do anything
		if (target == null) return;

		const element = document.querySelector(target);
		if (element == null) {
			throw new Error("Invalid state :: Element not found");
		}

		if (outerHTML) {
			element.outerHTML = content;
		} else if (element.tagName == "INPUT") {
			element.value = content;
		} else {
			element.innerHTML = content;
		}
		setTimeout(() => {
			scanListeners(element);
			document.addEventListener("keydown", (e) => nextTarget(e));
		}, 1);
	} catch (err) {
		console.error(err);
	}
}
function replaceError(err: ReplaceDirector | string) {
	console.error(err, application);
	try {
		if (err != null) {
			if (typeof err == "string") {
				document.body!.innerHTML =
					`<div id='error'><h1>${err}</h1><h2>Please reopen the application</h2></div>`;
			} else {
				document.querySelector(err.target!)!.innerHTML = err.content;
			}
		} else {document.body!.innerHTML =
				"<div style='position:fixed;inset:0'><h1 style='color:white'>Total error, reset application</h1></div>";}
	} catch (err) {
		console.error(err);
	}
}

function scanListeners(targetElement: Element | Document = document) {
	application = document.querySelector("#application");
	targetElement.querySelectorAll("[tx-goto]")
		.forEach((input) =>
			input.addEventListener("click", async function () {
				invoke<ReplaceDirector>(
					"page_x_" + input.getAttribute("tx-goto")!,
					input.hasAttribute("tx-id")
						? { id: input.getAttribute("tx-id") }
						: undefined,
				)
					.then(replaceContent)
					.catch(replaceError);
			})
		);
	targetElement.querySelectorAll("[tx-command]")
		//@ts-ignore incorrect warning over event listener
		.forEach((input) =>
			input.addEventListener(
				input.getAttribute("tx-trigger") ?? "click",
				function (
					event: Event & {
						target: HTMLFormElement | HTMLInputElement;
					},
				) {
					let data;
					switch (input.getAttribute("tx-trigger")) {
						case "input":
						case "change":
							data = { value: event.target?.value };
							break;
						case "submit": {
							event.preventDefault();
							data = {
								...Object.fromEntries(
									new FormData(<HTMLFormElement> event.target)
										.entries(),
								),
							};
							break;
						}
						default:
							if (input.hasAttribute("tx-id")) {
								data = {
									id: input.getAttribute("tx-id"),
								};
							}
					}

					invoke<{ target: string; content: string }>(
						input.getAttribute("tx-command")!,
						data,
					)
						.then((event) => {
							replaceContent(event);
						})
						.catch(replaceError);
				},
			)
		);
	targetElement.querySelectorAll("[tx-open]")
		.forEach((input) =>
			input.addEventListener("click", async function () {
				const target = targetElement.querySelector<HTMLDialogElement>(
					input.getAttribute("tx-open")!,
				);
				if (target == null) {
					throw new Error("Invalid state :: Element not found");
				}
				target.showModal();
			})
		);
	targetElement.querySelectorAll("[tx-close]")
		.forEach((input) =>
			input.addEventListener("click", async function () {
				const target = targetElement.querySelector<HTMLDialogElement>(
					input.getAttribute("tx-close")!,
				);
				if (target == null) {
					throw new Error("Invalid state :: Element not found");
				}
				target.close();
			})
		);
}

const goto_types = ["mark", "remark"];
/**
 * @param {InputEvent & {currentTarget: HTMLElement}} event
 * @returns
 */
function nextTarget(event: KeyboardEvent & { target: HTMLElement }) {
	if (
		(event.key != "Enter" && event.code != "Enter" && event.key != "Tab" &&
			event.code != "Tab") || event.target == null
	) return;
	event.preventDefault();

	const target = event.target;
	const line = event.target.closest("tr[data-index]");

	const index = parseInt(line.dataset.index!);
	const next_type = target.dataset.inputRole == goto_types[1]
		? goto_types[0]
		: goto_types[1];

	if (!index || !next_type) return;

	let next_index = index;
	// TODO implement both ways
	const preferenceDirection =
		document.querySelector<HTMLElement>("#scoresheet")
				?.dataset?.exerciseCommentLast
			? "mark"
			: "remark";
	if (next_type == (event.shiftKey ? goto_types[0] : goto_types[1])) {
		next_index = Math.max(1, next_index + (event.shiftKey ? -1 : 1));
	}

	let el: HTMLElement | null = document.querySelector(
		`[data-index="${next_index}"] .exercise-input[data-input-role="${next_type}"]`,
	);
	if (el == null) { // this is incase there is no next element for some reason
		const same_type = target.dataset.inputRole = goto_types[0]
			? goto_types[0]
			: goto_types[1];
		el = document.querySelector(
			`[data-index="${next_index}"] .exercise-input[data-input-role="${same_type}"]`,
		);
		if (el == null) { // here we select the emergency location if all else fails
			el = document.querySelector<HTMLTextAreaElement>("#final-remark")!;
		}
	}

	el?.focus();
}
