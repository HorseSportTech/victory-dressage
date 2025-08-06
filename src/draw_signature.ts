// deno-lint-ignore-file no-unused-vars no-window
// @ts-ignore-file
let ctx: CanvasRenderingContext2D;
let drawing = false;
let points: { x: number; y: number }[][] = [];

function signature_correctedCoordinates(
	e: PointerEvent & { currentTarget: EventTarget & HTMLCanvasElement },
): { x: number; y: number } {
	const rect = e.currentTarget.getBoundingClientRect();
	return {
		x: e.offsetX * (1000 / rect.width),
		y: e.offsetY * (500 / rect.height),
	};
}
// @ts-ignore is used in DOM
function signature_startDraw(
	e: PointerEvent & { currentTarget: EventTarget & HTMLCanvasElement },
) {
	e.stopPropagation();
	ctx = e.currentTarget.getContext("2d")!;
	drawing = true;
	points.push([signature_correctedCoordinates(e)]);
	ctx.beginPath();
	// @ts-ignore colour does in fact exist on context
	ctx.color = "black";
	ctx.lineWidth = 5;
}
// @ts-ignore is used in DOM
function signature_continueDraw(
	e: PointerEvent & { currentTarget: EventTarget & HTMLCanvasElement },
) {
	e.preventDefault();
	e.stopPropagation();
	if (!drawing) return;
	points[points.length - 1].push(signature_correctedCoordinates(e));
	ctx.lineTo(
		...(<[number, number]>Object.values(
			signature_correctedCoordinates(e),
		)),
	);
	ctx.stroke();
}
// @ts-ignore is used in DOM
function signature_endDraw(
	e: PointerEvent & { currentTarget: EventTarget & HTMLCanvasElement },
) {
	e.stopPropagation();
	if (!drawing) return;
	drawing = false;
	ctx.beginPath();
	points[points.length - 1].push(signature_correctedCoordinates(e));
	window.invoke("draw_signature", { pointLists: points })
		.catch((e: unknown) => console.error(e));
}

// @ts-ignore is used in dom
function signature_refresh() {
	ctx?.clearRect(0, 0, 1000, 500);
	points = [];
}
