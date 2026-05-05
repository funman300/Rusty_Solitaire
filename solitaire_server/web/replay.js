// Solitaire Quest replay viewer.
//
// Pulls the replay JSON from `/api/replays/:id`, hands it to the
// `solitaire_wasm` ReplayPlayer (which owns a real solitaire_core
// `GameState` compiled to WebAssembly), and renders each step's pile
// snapshot as plain HTML cards. The WASM module is the single source
// of truth for the rules engine — we don't re-implement Klondike in JS.

import init, { ReplayPlayer } from "/web/pkg/solitaire_wasm.js";

const STEP_INTERVAL_MS = 600;
const FAN_OFFSET_PX = 28;

const SUIT_GLYPHS = {
    clubs: "♣",
    diamonds: "♦",
    hearts: "♥",
    spades: "♠",
};

const RED_SUITS = new Set(["diamonds", "hearts"]);

const RANK_LABELS = [
    "", "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
];

const board = document.getElementById("board");
const captionEl = document.getElementById("caption");
const progressEl = document.getElementById("progress");
const scoreEl = document.getElementById("score");
const movesEl = document.getElementById("moves");
const resultEl = document.getElementById("result");
const btnPlay = document.getElementById("btn-play");
const btnStep = document.getElementById("btn-step");
const btnPrev = document.getElementById("btn-prev");

let player = null;
let replayJson = null;
let playInterval = null;

async function bootstrap() {
    // /replays/<id> — pull the id off the path so we can fetch the JSON.
    const id = window.location.pathname.split("/").pop();
    if (!id) {
        captionEl.textContent = "No replay id in URL.";
        return;
    }

    let response;
    try {
        response = await fetch(`/api/replays/${id}`);
    } catch (e) {
        captionEl.textContent = `Network error: ${e}`;
        return;
    }
    if (!response.ok) {
        captionEl.textContent = `Server returned ${response.status}.`;
        return;
    }
    const replay = await response.json();
    replayJson = JSON.stringify(replay);

    captionEl.textContent =
        `Seed ${replay.seed} · ${replay.draw_mode} · ${replay.mode} ` +
        `· ${formatDuration(replay.time_seconds)} win on ${replay.recorded_at} ` +
        `· final score ${replay.final_score}`;

    await init();
    resetPlayer();
}

function resetPlayer() {
    if (playInterval) {
        clearInterval(playInterval);
        playInterval = null;
        btnPlay.textContent = "▶ Play";
    }
    player = new ReplayPlayer(replayJson);
    btnPrev.disabled = true;
    btnStep.disabled = false;
    btnPlay.disabled = false;
    render(player.state());
}

function step() {
    const snap = player.step();
    if (snap === null) {
        finish();
        return null;
    }
    btnPrev.disabled = false;
    render(snap);
    return snap;
}

function finish() {
    if (playInterval) {
        clearInterval(playInterval);
        playInterval = null;
    }
    btnPlay.textContent = "▶ Play";
    btnPlay.disabled = true;
    btnStep.disabled = true;
}

function render(snap) {
    if (!snap) return;
    board.replaceChildren();
    renderPile("stock", snap.stock, false);
    renderPile("waste", snap.waste, false);
    snap.foundations.forEach((cards, idx) =>
        renderPile(`foundation-${idx}`, cards, false));
    snap.tableaus.forEach((cards, idx) =>
        renderPile(`tableau-${idx}`, cards, true));

    progressEl.textContent = `step ${snap.step_idx} / ${snap.total_steps}`;
    scoreEl.textContent = `Score ${snap.score}`;
    movesEl.textContent = `Moves ${snap.move_count}`;
    if (snap.is_won) {
        resultEl.textContent = "✨ Won";
        resultEl.classList.add("win");
    } else {
        resultEl.textContent = "";
        resultEl.classList.remove("win");
    }
}

function renderPile(name, cards, fan) {
    const pile = document.createElement("div");
    pile.className = `pile pile-${name}`;
    if (cards.length === 0) {
        const empty = document.createElement("div");
        empty.className = "pile-empty";
        pile.appendChild(empty);
        board.appendChild(pile);
        return;
    }
    cards.forEach((card, idx) => {
        const top = fan ? idx * FAN_OFFSET_PX : 0;
        pile.appendChild(buildCard(card, top));
    });
    board.appendChild(pile);
}

function buildCard(card, top) {
    const el = document.createElement("div");
    el.className = "card";
    el.style.top = `${top}px`;
    if (!card.face_up) {
        el.classList.add("face-down");
        return el;
    }
    el.classList.add(RED_SUITS.has(card.suit) ? "red" : "black");
    const label = RANK_LABELS[card.rank] || "?";
    const glyph = SUIT_GLYPHS[card.suit] || "?";

    const top_corner = document.createElement("span");
    top_corner.className = "corner top";
    top_corner.textContent = `${label}\n${glyph}`;
    el.appendChild(top_corner);

    const center = document.createElement("span");
    center.className = "center";
    center.textContent = glyph;
    el.appendChild(center);

    const bottom_corner = document.createElement("span");
    bottom_corner.className = "corner bottom";
    bottom_corner.textContent = `${label}\n${glyph}`;
    el.appendChild(bottom_corner);
    return el;
}

function formatDuration(seconds) {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${String(s).padStart(2, "0")}`;
}

btnStep.addEventListener("click", () => {
    if (player) step();
});

btnPlay.addEventListener("click", () => {
    if (!player) return;
    if (playInterval) {
        clearInterval(playInterval);
        playInterval = null;
        btnPlay.textContent = "▶ Play";
        return;
    }
    btnPlay.textContent = "⏸ Pause";
    playInterval = setInterval(() => {
        const snap = step();
        if (snap === null) finish();
    }, STEP_INTERVAL_MS);
});

btnPrev.addEventListener("click", () => {
    if (replayJson) resetPlayer();
});

bootstrap();
