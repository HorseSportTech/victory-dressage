let autoSign = false;

let ctx: CanvasRenderingContext2D;
let drawing = false;
let points: { x: number; y: number }[][] = [];
let svgPath = "";

function signature_correctedCoordinates(
	e: PointerEvent & { currentTarget: EventTarget & HTMLCanvasElement },
): { x: number; y: number } {
	const rect = e.currentTarget.getBoundingClientRect();
	return {
		x: e.offsetX * (1000 / rect.width),
		y: e.offsetY * (500 / rect.height),
	};
}
function signature_startDraw(
	e: PointerEvent & { currentTarget: EventTarget & HTMLCanvasElement },
) {
	e.stopPropagation();
	ctx = e.currentTarget.getContext("2d")!;
	drawing = true;
	points.push([signature_correctedCoordinates(e)]);
	ctx.beginPath();
	// @ts-ignore
	ctx.color = "black";
	ctx.lineWidth = 5;
}
function signature_continueDraw(
	e: PointerEvent & { currentTarget: EventTarget & HTMLCanvasElement },
) {
	e.preventDefault();
	e.stopPropagation();
	if (!drawing) return;
	points[points.length - 1].push(signature_correctedCoordinates(e));
	ctx.lineTo(
		...(<[number, number]> Object.values(
			signature_correctedCoordinates(e),
		)),
	);
	ctx.stroke();
}
function signature_endDraw(
	e: PointerEvent & { currentTarget: EventTarget & HTMLCanvasElement },
) {
	e.stopPropagation();
	if (!drawing) return;
	drawing = false;
	ctx.beginPath();
	points[points.length - 1].push(signature_correctedCoordinates(e));
	window.invoke("draw_signature", { pointLists: points })
		.then((e: string) => {
			svgPath = e;
			console.log(e);
		})
		.catch((e: unknown) => console.error(e));
}

function signature_refresh() {
	ctx?.clearRect(0, 0, 1000, 500);
	points = [];
	svgPath = "";
}

