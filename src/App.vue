<script setup lang="ts">
import {onMounted, onUnmounted, Ref, ref, watch} from "vue";

interface Point {
  x: number,
  y: number,
}

let renderInterval: NodeJS.Timeout|null = null;

const canvas: Ref<HTMLCanvasElement | null> = ref(null);
const context: Ref<CanvasRenderingContext2D | null> = ref(null);

let pixelsPerUnit = 100;

let dragging = false;
let lastMouse: Point = {x: 0, y: 0};

let offsetValues: Point = {x: 0, y: 0};

const downscaleFactor = 2.0;

let joints: Point[] = [];

const moveFactor = 2.0;

const zoomInFactor = 0.98;
const zoomOutFactor = 1.02;

const simpsonStart = ref(1);
const simpsonEnd = ref(5);
const simpsonDivisions = ref(1);

//----------------------------------------------------
// USER INPUT
//----------------------------------------------------

function startDrag(e: MouseEvent) {
  lastMouse.x = e.x;
  lastMouse.y = e.y;
  renderInterval = setInterval(rerender, 10);
  dragging = true;
}

function stopDrag() {
  clearInterval(renderInterval!);
  rerender();
  dragging = false;
}

function dragGraph(e: MouseEvent) {
  if (!dragging) return;

  let deltaX = e.x - lastMouse.x;
  let deltaY = e.y - lastMouse.y;
  lastMouse.x = e.x;
  lastMouse.y = e.y;

  offsetValues.x += deltaX * moveFactor;
  offsetValues.y += deltaY * moveFactor;
}

function zoomGraph(e: WheelEvent) {
  if (e.deltaY > 0) {
    pixelsPerUnit *= zoomOutFactor;
  } else {
    pixelsPerUnit *= zoomInFactor;
  }
  if (pixelsPerUnit < 1) {
    pixelsPerUnit = 0.1;
  }

  rerender();
}

//----------------------------------------------------
// RENDERING
//----------------------------------------------------

onMounted(() => {
  window.addEventListener("resize", rerender);
  rerender()
})

onUnmounted(() => {
  window.removeEventListener("resize", rerender);
})

watch([simpsonStart, simpsonEnd, simpsonDivisions], () => {
  rerender();
})

function rerender() {
  configureContext();
  renderGraph();
}

function configureContext() {
  canvas.value!.width = canvas.value!.clientWidth * downscaleFactor;
  canvas.value!.height = canvas.value!.clientHeight * downscaleFactor;
  context.value = canvas.value?.getContext("2d")!;
  context.value!.lineWidth = 3;
  context.value!.lineCap = "round";
}

function renderGraph() {
  clearContext();
  drawAxis();
  drawGrid();

  calculateGraph();

  context.value!.beginPath();
  for (let i = 0; i < joints.length - 1; i++) {
    batchDrawLineFromTo(joints[i], joints[i + 1])
  }

  context.value!.strokeStyle = "white"
  context.value!.lineWidth = 3;
  context.value!.stroke();

  context.value!.closePath();

  drawSimpson();
}

function drawSimpson() {
  if (simpsonStart.value === 0 && simpsonEnd.value === 0) return;

  context.value!.beginPath();

  const simpsonPartLength = (simpsonEnd.value - simpsonStart.value) / (simpsonDivisions.value + 1)
  for (let i = 0; i < simpsonDivisions.value + 2; i++) {
    const simpsonX = simpsonStart.value + i * simpsonPartLength;
    let simpsonPartStart: Point = { x: unitsToPixels(simpsonX), y: 0 };
    let simpsonPartEnd: Point = { x: unitsToPixels(simpsonX), y: unitsToPixels(y(simpsonX)) };

    batchDrawLineFromTo(simpsonPartStart, simpsonPartEnd)
  }

  context.value!.lineWidth = 1;
  context.value!.strokeStyle = "rgba(144,238,144,0.5)"
  context.value!.stroke();
  context.value!.closePath();
}

function drawAxis() {
  const xStart: Point = correctPointOrigin(xStartPoint());
  const xEnd: Point = correctPointOrigin(xEndPoint());

  const yStart: Point = correctPointOrigin(yStartPoint());
  const yEnd: Point = correctPointOrigin(yEndPoint());

  context.value!.beginPath();
  context.value!.moveTo(xStart.x, xStart.y);
  context.value!.lineTo(xEnd.x, xEnd.y);

  context.value!.moveTo(yStart.x, yStart.y);
  context.value!.lineTo(yEnd.x, yEnd.y);

  context.value!.strokeStyle = "white"
  context.value!.lineWidth = 3;
  context.value!.stroke();
  context.value!.closePath();
}

function drawGrid() {
  const xStart: Point = xStartPoint();
  const xEnd: Point = xEndPoint();

  const yStart: Point = yStartPoint();
  const yEnd: Point = yEndPoint();

  const offsetY = Math.trunc(offsetValues.y / pixelsPerUnit) * pixelsPerUnit;
  const offsetX = Math.trunc(offsetValues.x / pixelsPerUnit) * pixelsPerUnit;

  xStart.y += offsetY;
  xEnd.y += offsetY;

  yStart.x -= offsetX;
  yEnd.x -= offsetX;

  context.value!.beginPath();
  for (let i = 0; i < canvas.value!.height / pixelsPerUnit; i++) {
    const correctedXStart = correctPointOrigin(xStart);
    const correctedXEnd = correctPointOrigin(xEnd);

    context.value!.moveTo(correctedXStart.x, correctedXStart.y);
    context.value!.lineTo(correctedXEnd.x, correctedXEnd.y);

    xStart.y += unitsToPixels(1);
    xEnd.y += unitsToPixels(1);
  }

  for (let i = 0; i < canvas.value!.width / pixelsPerUnit; i++) {
    const correctedYStart = correctPointOrigin(yStart);
    const correctedYEnd = correctPointOrigin(yEnd);

    context.value!.moveTo(correctedYStart.x, correctedYStart.y);
    context.value!.lineTo(correctedYEnd.x, correctedYEnd.y);

    yStart.x += unitsToPixels(1);
    yEnd.x += unitsToPixels(1);
  }

  context.value!.lineWidth = 1;
  context.value!.strokeStyle = "rgba(255.0,255.0,255.0,0.1)"
  context.value!.stroke();
  context.value!.closePath();
}

function xStartPoint(): Point {
  return {x: 0, y: -offsetValues.y};
}

function xEndPoint(): Point {
  return {x: canvas.value!.width, y: -offsetValues.y};
}

function yStartPoint(): Point {
  return {x: offsetValues.x, y: canvas.value!.height};
}

function yEndPoint(): Point {
  return {x: offsetValues.x, y: 0};
}

function calculateGraph() {
  joints = [];

  for (let i = -offsetValues.x; i < -offsetValues.x + canvas.value!.width; i++) {
    joints.push({
      x: i,
      y: unitsToPixels(y(pixelsToUnit(i))),
    })
  }
}

function batchDrawLineFromTo(from: Point, to: Point) {
  const correctFrom = offset(correctPointOrigin(from));
  const correctTo = offset(correctPointOrigin(to));

  if (isOutsideView(correctFrom) && isOutsideView(correctTo)) return;

  context.value!.moveTo(correctFrom.x, correctFrom.y);
  context.value!.lineTo(correctTo.x, correctTo.y);
}

function clearContext() {
  context.value!.clearRect(0, 0, canvas.value!.width, canvas.value!.height);
  context.value!.beginPath();
}

function y(x: number): number {
  return Math.sin(x / 5) - Math.sin(x/ 2) + 2;
}

//----------------------------------------------------
// UTIL
//----------------------------------------------------

function isOutsideView(point: Point): boolean {
  return point.x < 0 || point.y < 0 || point.x > canvas.value!.width || point.y > canvas.value!.height;
}

function correctPointOrigin(point: Point): Point {
  return {
    x: point.x,
    y: canvas.value!.height - point.y,
  }
}

function offset(point: Point): Point {
  return {
    x: point.x + offsetValues.x,
    y: point.y + offsetValues.y,
  };
}

function pixelsToUnit(pixels: number): number {
  return pixels / pixelsPerUnit;
}

function unitsToPixels(units: number): number {
  return units * pixelsPerUnit;
}
</script>

<template>
  <header id="header" class="d-flex justify-content-center align-items-center gap-3">
    <label class="h5 m-0">Function:</label>
    <input id="function-input" type="text" class="form-control form-control-sm" placeholder="f(x) = ">
  </header>
  <div id="menu" class="d-flex align-items-center justify-content-around">
    <div class="d-flex align-items-center gap-2 ">
      <label>Subdivisions: </label>
      <input type="number" inputmode="numeric" v-model.number="simpsonDivisions" class="form-control form-control-sm option">
    </div>
    <div class="d-flex gap-3">
      <div class="d-flex align-items-center gap-2 ">
        <label>From: </label>
        <input type="number" inputmode="numeric" v-model.number="simpsonStart" class="form-control form-control-sm option">
      </div>
      <div class="d-flex align-items-center gap-2 ">
        <label>To: </label>
        <input type="number" inputmode="numeric" v-model.number="simpsonEnd" class="form-control form-control-sm option">
      </div>
    </div>
  </div>
  <div id="graph-container" class="d-flex justify-content-center align-items-center">
    <canvas id="graph" ref="canvas" @mousedown="startDrag" @mouseup="stopDrag" @mouseleave="stopDrag"
            @mousemove="dragGraph" @wheel.prevent="zoomGraph">

    </canvas>
  </div>
</template>

<style scoped lang="scss">
@import "bootstrap/scss/functions";
@import "bootstrap/scss/variables";
@import "bootstrap/scss/variables-dark";

#header {
  height: 50px;
  background: darken($body-bg-dark, 2.5%);
}

#function-input {
  width: 500px;
}

#menu {
  height: 50px;

  width: 90vw;

  margin: 10px 5vw;

  border-radius: 7px;

  background: darken($body-bg-dark, 2.5%);
}

.option {
  width: 75px;
}

#graph-container {
  width: 90vw;
  height: calc(100vh - 50px - 10px - 50px - 10px - 20px);

  margin: 0 5vw;

  border-radius: 7px;

  background: darken($body-bg-dark, 2.5%);
}

#graph {
  width: 90%;
  height: 90%;

  background: lighten($body-bg-dark, 2%);
}
</style>
