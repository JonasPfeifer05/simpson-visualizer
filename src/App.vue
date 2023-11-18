<script setup lang="ts">
import {onMounted, Ref, ref} from "vue";

const canvas: Ref<HTMLCanvasElement|null> = ref(null);
const context: Ref<CanvasRenderingContext2D|null> = ref(null);

let dragging = false;
let lastMouseX = -1;
let lastMouseY = -1;


let offsetX = 0;
let offsetY = 0;

const canvasWidth = 1920;
const canvasHeight = 1080;
let scale = 1.0;

function startDrag(e: MouseEvent) {
  dragging = true;
  lastMouseX = e.x;
  lastMouseY = e.y;
}

function stopDrag() {
  dragging = false;
}

function dragGraph(e: MouseEvent) {
  if (!dragging) return;

  let deltaX = e.x - lastMouseX;
  let deltaY = e.y - lastMouseY;
  lastMouseX = e.x;
  lastMouseY = e.y;

  offsetX += deltaX * 2;
  offsetY += deltaY * 2;

  renderGraph();
}

function zoomGraph(e: WheelEvent) {
  scale += e.deltaY / 1000;
  configureContext();
  renderGraph();
}

onMounted(() => {
  configureContext();
  renderGraph();
})

function renderGraph() {
  clear();
  drawLineFromTo(0,1080 + offsetY,1920,1080 + offsetY );
  drawLineFromTo(offsetX,0,offsetX,1080);
  for (let i = 0; i < 1000; i++) {
    context.value!.beginPath();
    context.value!.moveTo(i + offsetX, canvasHeight - Math.pow(i / 100.0, 2) * 10 + offsetY);
    context.value!.lineTo((i + 1) + offsetX, canvasHeight - Math.pow((i + 1) / 100.0, 2) * 10 + offsetY);
    context.value!.stroke();
  }

}
function drawLineFromTo(fromX: number, fromY: number, toX: number, toY: number) {
  context.value!.moveTo(fromX, fromY);
  context.value!.lineTo(toX, toY);
  context.value!.stroke();
}

function clear() {
  context.value!.clearRect(0,0, canvas.value!.width, canvas.value!.height);
  context.value!.beginPath();
}

function configureContext() {
  canvas.value!.width = canvasWidth;
  canvas.value!.height = canvasHeight;
  context.value = canvas.value?.getContext("2d")!;
  context.value!.lineWidth = 3;
  context.value!.lineCap = "round";
  context.value!.strokeStyle = "rgb(255,255,255)"
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
      <input type="number" inputmode="numeric" class="form-control form-control-sm option">
    </div>
    <div class="d-flex gap-3">
      <div class="d-flex align-items-center gap-2 ">
        <label>From: </label>
        <input type="number" inputmode="numeric" class="form-control form-control-sm option">
      </div>
      <div class="d-flex align-items-center gap-2 ">
        <label>To: </label>
        <input type="number" inputmode="numeric" class="form-control form-control-sm option">
      </div>
    </div>
  </div>
  <div id="graph-container" class="d-flex justify-content-center align-items-center">
    <canvas id="graph" ref="canvas" @mousedown="startDrag" @mouseup="stopDrag" @mouseleave="stopDrag" @mousemove="dragGraph" @wheel="zoomGraph">

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
