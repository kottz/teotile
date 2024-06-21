import init, { GameWrapper } from './pkg/teotile_web.js';

let game;
let canvas;
let ctx;
const GRID_SIZE = 12;
const CELL_SIZE = 30;

async function initialize() {
    await init();
    game = new GameWrapper();
    canvas = document.getElementById('gameCanvas');
    ctx = canvas.getContext('2d');
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    setupMobileGamepad();
    requestAnimationFrame(gameLoop);
}

function setupMobileGamepad() {
    const gamepadButtons = document.querySelectorAll('.gamepad-btn');
    gamepadButtons.forEach(button => {
        button.addEventListener('touchstart', (e) => {
            e.preventDefault();
            const key = button.getAttribute('data-key');
		console.log(key);
            processInput({ key }, 0);
        });
        button.addEventListener('touchend', (e) => {
            e.preventDefault();
            const key = button.getAttribute('data-key');
            processInput({ key }, 1);
        });
    });
}

function handleKeyDown(event) {
    processInput(event, 0); // 0 for ButtonState::Pressed
}

function handleKeyUp(event) {
    processInput(event, 1); // 1 for ButtonState::Released
}

function processInput(event, buttonState) {
    let commandType, player;
    switch (event.key) {
        case 'w': commandType = 0; player = 0; break;
        case 'a': commandType = 2; player = 0; break;
        case 's': commandType = 1; player = 0; break;
        case 'd': commandType = 3; player = 0; break;
        case 'e':
        case 'r': commandType = 4; player = 0; break;
        case 'q':
        case 'f': commandType = 5; player = 0; break;
        case 'ArrowUp': commandType = 0; player = 1; break;
        case 'ArrowLeft': commandType = 2; player = 1; break;
        case 'ArrowDown': commandType = 1; player = 1; break;
        case 'ArrowRight': commandType = 3; player = 1; break;
        case 'Enter':
        case 'm': commandType = 4; player = 1; break;
        case 'Backspace': commandType = 5; player = 1; break;
        default: return;
    }
    game.process_input(commandType, buttonState, player);
}

let lastTime = 0;
function gameLoop(timestamp) {
    const delta = (timestamp - lastTime) / 1000;
    lastTime = timestamp;
    game.update(delta);
    render();
    requestAnimationFrame(gameLoop);
}

function render() {
    const pixelData = game.render();
    const imageData = ctx.createImageData(GRID_SIZE, GRID_SIZE);
    for (let y = 0; y < GRID_SIZE; y++) {
        for (let x = 0; x < GRID_SIZE; x++) {
            const rotatedIndex = ((GRID_SIZE - 1 - x) * GRID_SIZE + y) * 4;
            const originalIndex = (y * GRID_SIZE + x) * 3;
            imageData.data[rotatedIndex] = pixelData[originalIndex];
            imageData.data[rotatedIndex + 1] = pixelData[originalIndex + 1];
            imageData.data[rotatedIndex + 2] = pixelData[originalIndex + 2];
            imageData.data[rotatedIndex + 3] = 255;
        }
    }
    ctx.putImageData(imageData, 0, 0);
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(canvas, 0, 0, GRID_SIZE, GRID_SIZE, 0, 0, CELL_SIZE * GRID_SIZE, CELL_SIZE * GRID_SIZE);
}

initialize();
