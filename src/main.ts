import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from '@tauri-apps/api/event'
let application: HTMLElement | null;
declare global { interface Window { invoke: any; debounce: (callback: (...args: unknown[]) => unknown, wait: number) => (...args: unknown[]) => void } }
window.invoke = invoke;
window.debounce = (callback: (...args:unknown[]) => unknown, wait: number) => {
	let timeoutId: undefined|number = undefined;
	return (...args: unknown[]) => {
		window.clearTimeout(timeoutId);
		timeoutId = window.setTimeout(() => callback(...args), wait);
	};
  }

type ReplaceDirector = {target?: string, content: string};

const unlisten = listen<{target?:string, content: string}>("page_update",async  (event) => {
	console.log(event)
	replaceContent(event.payload)
})

window.addEventListener("DOMContentLoaded", () => {
	application = document.querySelector("#application");
	if (application == null) {
		document.firstElementChild!.innerHTML = "<h1>Oh no</h1>";
		throw new Error("Invalid state :: Application not found");
	}
	invoke<ReplaceDirector>("page_x_current")
		.then(replaceContent)
		.catch(replaceError);
})

function replaceContent({target, content}: ReplaceDirector): void {
	try {
		// no target returned. Don't need to do anything
		if (target == null) return;
	
		const element = document.querySelector(target);
		if (element == null) throw new Error("Invalid state :: Element not found");
		// const frag = document.createElement("template");
		// frag.innerHTML = content;
		element.innerHTML = content;
		// element.appendChild(frag.content)
		setTimeout(() => scanListeners(element), 1);
	} catch (err) {
		console.error(err);
	}
}
function replaceError(err: ReplaceDirector | string) {
	console.error(err, application)
	try {
		if (err != null) {
			if (typeof err == "string") {
				document.body!.innerHTML = err;
			} else {
				document.querySelector(err.target!)!.innerHTML = err.content
			}
		} else {document.body!.innerHTML = "<h1 style='color:white'>Total error, reset application</h1>"}
	} catch (err) {
		console.error(err)
	}
}

function scanListeners(targetElement: Element|Document = document) {
	application = document.querySelector("#application");
	targetElement.querySelectorAll("[tx-goto]")
		.forEach(input => input.addEventListener("click", async function () {
			invoke<ReplaceDirector>(
				"page_x_"+input.getAttribute("tx-goto")!,
				input.hasAttribute("tx-id") ? {id: input.getAttribute("tx-id") } : undefined,
			)
			.then(replaceContent)
			.catch(replaceError)
		}));
	targetElement.querySelectorAll("[tx-command]")
		//@ts-ignore incorrect warning over event listener
		.forEach(input => input.addEventListener(
			input.getAttribute("tx-trigger") ?? "click",
			function (event: Event & {target: HTMLFormElement|HTMLInputElement}) {

			let data;
			switch (input.getAttribute("tx-trigger")) {
				case "input":
				case "change":
					data = { value: event.target?.value }; break;
				case "submit": {
					event.preventDefault();
					data = {...Object.fromEntries(new FormData(<HTMLFormElement>event.target).entries())};
					break;
				}
				default: 
					if (input.hasAttribute("tx-id")) data = {id: input.getAttribute("tx-id")};
			}

			console.log("command", input.getAttribute("tx-command"), data)
			invoke<{target:string, content:string}>(
				input.getAttribute("tx-command")!,
				data,
			)
			.then(replaceContent)
			.catch(replaceError)
		}));
	targetElement.querySelectorAll("[tx-open]")
		.forEach(input => input.addEventListener("click", async function () {
			const target = targetElement.querySelector<HTMLDialogElement>(input.getAttribute("tx-open")!);
			if (target == null) throw new Error("Invalid state :: Element not found");
			target.showModal()
		}));
	targetElement.querySelectorAll("[tx-close]")
		.forEach(input => input.addEventListener("click", async function () {
			const target = targetElement.querySelector<HTMLDialogElement>(input.getAttribute("tx-close")!);
			if (target == null) throw new Error("Invalid state :: Element not found");
			target.close()
		}));
}
