// ─── Carcassonne Web UI — App Controller ────────────────────────────────────
// Bridges JavaScript UI with Rust/WASM game engine
// ─────────────────────────────────────────────────────────────────────────────

import init, { WasmGame, init_panic_hook } from "./pkg/carcossonne_uct.js";

// ─── Constants ───────────────────────────────────────────────────────────────

const TILE_SIZE_PX = 80;
const TILE_IMAGES_DIR = "tilesets_images";
const BOT_DELAY_MS = 800;

// All 24 tile types
const ALL_TILE_NAMES = [
    "CCCC_PENNANT", "CCFC", "CCFC_PENNANT", "CCFF", "CCFF_DISCONNECTED",
    "CCFF_PENNANT", "CCRC", "CCRC_PENNANT", "CCRR", "CCRR_PENNANT",
    "CFCF", "CFCF_DISCONNECTED", "CFCF_PENNANT", "CFFF", "CFRR",
    "CRFR", "CRRF", "CRRR_DISCONNECTED", "FFFF_CLOISTER", "FFRF_CLOISTER",
    "FFRR", "FRFR", "FRRR_DISCONNECTED", "RRRR"
];

// ─── State ───────────────────────────────────────────────────────────────────

let wasmGame = null;
let gameState = {
    selectedTile: null,
    rotation: 0,
    phase: "idle",         // idle | selecting | placing | meeple | bot | gameover
    humanMoves: null,      // cached from WASM
    meepleOptions: null,   // cached from WASM
    pendingPlacement: null, // {x, y} of tile just placed, awaiting meeple
    placedTiles: [],       // visual record of all placed tiles
};

let playerName = null;

// Board panning state
let camera = {
    x: 0,
    y: 0,
    dragging: false,
    startX: 0,
    startY: 0,
    startCamX: 0,
    startCamY: 0,
};

// ─── DOM References ──────────────────────────────────────────────────────────

const dom = {};

function cacheDom() {
    dom.loadingOverlay = document.getElementById("loading-overlay");
    dom.mainMenu = document.getElementById("main-menu");
    dom.gameScreen = document.getElementById("game-screen");
    dom.btnPlayBot = document.getElementById("btn-play-bot");
    dom.botTypeSelect = document.getElementById("bot-type");
    dom.plainModeCheckbox = document.getElementById("plain-mode");
    dom.boardContainer = document.getElementById("board-container");
    dom.boardWrapper = document.getElementById("board-wrapper");
    dom.tileGrid = document.getElementById("tile-grid");
    dom.previewContainer = document.getElementById("preview-container");
    dom.previewImage = document.getElementById("preview-image");
    dom.previewPlaceholder = document.getElementById("preview-placeholder");
    dom.btnRotate = document.getElementById("btn-rotate");
    dom.rotationLabel = document.getElementById("rotation-label");
    dom.btnSkipMeeple = document.getElementById("btn-skip-meeple");
    dom.scoreHumanValue = document.getElementById("score-human-value");
    dom.scoreBotValue = document.getElementById("score-bot-value");
    dom.followersHuman = document.getElementById("followers-human");
    dom.followersBot = document.getElementById("followers-bot");
    dom.turnIndicator = document.getElementById("turn-indicator");
    dom.statusMessage = document.getElementById("status-message");
    dom.botThinking = document.getElementById("bot-thinking");
    dom.gameOverModal = document.getElementById("game-over-modal");
    dom.gameOverTitle = document.getElementById("game-over-title");
    dom.winnerCrown = document.getElementById("winner-crown");
    dom.finalHumanScore = document.getElementById("final-human-score");
    dom.finalBotScore = document.getElementById("final-bot-score");
    dom.btnPlayAgain = document.getElementById("btn-play-again");
    dom.scoreHuman = document.getElementById("score-human");
    dom.scoreBot = document.getElementById("score-bot");
    dom.playerName = document.getElementById("player-name");
}

// ─── Initialization ──────────────────────────────────────────────────────────

async function main() {
    cacheDom();
    await init();
    init_panic_hook();
    dom.loadingOverlay.classList.add("hidden");
    setupEventListeners();
}

function setupEventListeners() {
    dom.btnPlayBot.addEventListener("click", startGame);
    dom.btnRotate.addEventListener("click", rotateTile);
    dom.btnSkipMeeple.addEventListener("click", skipMeeple);
    dom.btnPlayAgain.addEventListener("click", () => {
        dom.gameOverModal.classList.remove("active");
        startGame();
    });

    // Board panning
    dom.boardContainer.addEventListener("mousedown", onBoardMouseDown);
    dom.boardContainer.addEventListener("mousemove", onBoardMouseMove);
    dom.boardContainer.addEventListener("mouseup", onBoardMouseUp);
    dom.boardContainer.addEventListener("mouseleave", onBoardMouseUp);

    // Prevent context menu on board
    dom.boardContainer.addEventListener("contextmenu", (e) => e.preventDefault());
}

// ─── Game Lifecycle ──────────────────────────────────────────────────────────

function startGame() {
    if(!dom.playerName.value) {
        alert("Choose a pseudonym");
        return;
    }
    playerName = dom.playerName.value;

    const plainMode = dom.plainModeCheckbox ? dom.plainModeCheckbox.checked : false;
    // const botDepth = dom.botDepthSelect ? parseInt(dom.botDepthSelect.value, 10) : 1;
    wasmGame = new WasmGame(plainMode, dom.botTypeSelect.value);
    gameState = {
        selectedTile: null,
        rotation: 0,
        phase: "idle",
        humanMoves: null,
        meepleOptions: null,
        pendingPlacement: null,
        placedTiles: [],
    };

    // Show game screen
    dom.mainMenu.style.display = "none";
    dom.gameScreen.classList.add("active");
    dom.gameOverModal.classList.remove("active");

    // Clear board
    dom.boardWrapper.innerHTML = "";

    // Get starting info and render
    const startInfo = JSON.parse(wasmGame.get_starting_info());
    const boardState = JSON.parse(wasmGame.get_board_state());

    gameState.placedTiles = boardState.placed_tiles;

    // Center camera on starting tile
    const containerRect = dom.boardContainer.getBoundingClientRect();
    camera.x = containerRect.width / 2 - TILE_SIZE_PX / 2;
    camera.y = containerRect.height / 2 - TILE_SIZE_PX / 2;

    renderBoard();
    updateScores(boardState.scores);
    setStatus("Select a tile to place");
    startHumanTurn();
}

// ─── Turn Management ─────────────────────────────────────────────────────────

function startHumanTurn() {
    gameState.phase = "selecting";
    gameState.selectedTile = null;
    gameState.rotation = 0;
    gameState.meepleOptions = null;
    gameState.pendingPlacement = null;

    // Update turn indicator
    dom.turnIndicator.textContent = "Your Turn";
    dom.turnIndicator.className = "turn-indicator human-turn";
    dom.scoreHuman.classList.add("active");
    dom.scoreBot.classList.remove("active");

    // Compute valid moves
    const movesJson = wasmGame.compute_human_moves();
    gameState.humanMoves = JSON.parse(movesJson);

    if (gameState.humanMoves.game_over) {
        endGame();
        return;
    }

    // Render tile selector
    renderTileGrid();
    clearSelection();
    clearMarkers();
    setStatus("Select a tile to place");
}

function startBotTurn() {
    gameState.phase = "bot";
    dom.turnIndicator.textContent = "Bot's Turn";
    dom.turnIndicator.className = "turn-indicator bot-turn";
    dom.scoreHuman.classList.remove("active");
    dom.scoreBot.classList.add("active");
    dom.botThinking.classList.add("active");
    setStatus("Bot is thinking…");

    // Disable tile interactions
    clearSelection();
    clearMarkers();

    setTimeout(() => {
        const resultJson = wasmGame.bot_play();
        const result = JSON.parse(resultJson);

        dom.botThinking.classList.remove("active");

        if (result.no_moves || result.game_over) {
            endGame();
            return;
        }

        // Record the bot's placement
        const botTile = {
            tile_name: result.tile_name,
            x: result.x,
            y: result.y,
            rotation: result.rotation,
            meeples: result.meeple ? [result.meeple] : [],
        };
        gameState.placedTiles.push(botTile);

        // Render the new tile
        renderBoard();
        scrollToTile(result.x, result.y);
        updateScores(result.scores);

        setStatus(`Bot placed ${formatTileName(result.tile_name)}`);

        if (result.game_over) {
            endGame();
            return;
        }

        // Update tile grid counts
        renderTileGrid();

        // Back to human turn
        setTimeout(() => startHumanTurn(), 500);
    }, BOT_DELAY_MS);
}

function sendDB() {
    const matchData = {
        player_name: "Alice",
        player_score: 10,
        opponent_score: 5
    };

    // Send the data via a POST request
    fetch('save_game.php', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(matchData)
    })
    .then(response => response.json())
    .then(data => {
        if (data.success) {
            console.log('Match saved successfully!', data);
        } else {
            console.error('Error saving match:', data.error);
        }
    })
    .catch(error => console.error('Network error:', error));
}

function endGame() {
    gameState.phase = "gameover";
    const resultJson = wasmGame.end_game();
    const result = JSON.parse(resultJson);

    updateScores(result.scores);

    dom.finalHumanScore.textContent = result.scores.human;
    dom.finalBotScore.textContent = result.scores.bot;

    if (result.winner === "human") {
        dom.winnerCrown.textContent = "🏆";
        dom.gameOverTitle.textContent = "You Win!";
    } else if (result.winner === "bot") {
        dom.winnerCrown.textContent = "🤖";
        dom.gameOverTitle.textContent = "Bot Wins!";
    } else {
        dom.winnerCrown.textContent = "🤝";
        dom.gameOverTitle.textContent = "It's a Tie!";
    }

    dom.gameOverModal.classList.add("active");
    setStatus("Game Over");
}

// ─── Tile Selection ──────────────────────────────────────────────────────────

function selectTile(tileName) {
    if (gameState.phase !== "selecting" && gameState.phase !== "placing") return;

    gameState.selectedTile = tileName;
    gameState.rotation = 0;
    gameState.phase = "placing";

    // Update preview
    dom.previewImage.src = `${TILE_IMAGES_DIR}/${tileName}.png`;
    dom.previewImage.style.display = "block";
    dom.previewImage.style.transform = "rotate(0deg)";
    dom.previewPlaceholder.style.display = "none";
    dom.btnRotate.style.display = "block";
    dom.rotationLabel.textContent = "0°";

    // Highlight selected tile in grid
    document.querySelectorAll(".tile-card").forEach(card => {
        card.classList.toggle("selected", card.dataset.tileName === tileName);
    });

    // Show valid placements
    showValidPlacements();
    setStatus(`Place ${formatTileName(tileName)} on the board`);
}

function rotateTile() {
    if (!gameState.selectedTile) return;

    gameState.rotation = (gameState.rotation + 3) % 4;
    const degrees = ((4 - gameState.rotation) % 4) * 90;
    dom.previewImage.style.transform = `rotate(${degrees}deg)`;
    dom.rotationLabel.textContent = `${degrees}°`;

    // Update valid placements for new rotation
    showValidPlacements();
}

function clearSelection() {
    gameState.selectedTile = null;
    gameState.rotation = 0;
    dom.previewImage.style.display = "none";
    dom.previewPlaceholder.style.display = "block";
    dom.btnRotate.style.display = "none";
    dom.rotationLabel.textContent = "";
    dom.btnSkipMeeple.classList.remove("visible");

    document.querySelectorAll(".tile-card.selected").forEach(card => {
        card.classList.remove("selected");
    });
}

// ─── Board Rendering ─────────────────────────────────────────────────────────

function renderBoard() {
    dom.boardWrapper.innerHTML = "";

    for (const tile of gameState.placedTiles) {
        const tileEl = createTileElement(tile);
        dom.boardWrapper.appendChild(tileEl);
    }

    updateBoardTransform();
}

function createTileElement(tile) {
    const el = document.createElement("div");
    el.className = "board-tile";
    el.style.left = `${tile.x * TILE_SIZE_PX}px`;
    el.style.top = `${tile.y * TILE_SIZE_PX}px`;

    const img = document.createElement("img");
    img.src = `${TILE_IMAGES_DIR}/${tile.tile_name}.png`;
    img.alt = tile.tile_name;
    img.style.transform = `rotate(${((4 - tile.rotation) % 4) * 90}deg)`;
    img.draggable = false;
    el.appendChild(img);

    // Add meeple markers
    if (tile.meeples) {
        for (const meeple of tile.meeples) {
            const meepleEl = document.createElement("div");
            meepleEl.className = `meeple-marker ${meeple.player}`;
            meepleEl.style.left = `${meeple.frac_x * 100}%`;
            meepleEl.style.top = `${meeple.frac_y * 100}%`;
            meepleEl.title = `${meeple.player} - ${meeple.structure_type}`;
            el.appendChild(meepleEl);
        }
    }

    return el;
}

function updateBoardTransform() {
    dom.boardWrapper.style.transform = `translate(${camera.x}px, ${camera.y}px)`;
}

function scrollToTile(x, y) {
    const containerRect = dom.boardContainer.getBoundingClientRect();
    const targetX = containerRect.width / 2 - (x * TILE_SIZE_PX + TILE_SIZE_PX / 2);
    const targetY = containerRect.height / 2 - (y * TILE_SIZE_PX + TILE_SIZE_PX / 2);

    // Animate camera
    const startX = camera.x;
    const startY = camera.y;
    const duration = 300;
    const startTime = performance.now();

    function animate(currentTime) {
        const elapsed = currentTime - startTime;
        const progress = Math.min(elapsed / duration, 1);
        const eased = 1 - Math.pow(1 - progress, 3); // ease-out cubic

        camera.x = startX + (targetX - startX) * eased;
        camera.y = startY + (targetY - startY) * eased;
        updateBoardTransform();

        if (progress < 1) {
            requestAnimationFrame(animate);
        }
    }
    requestAnimationFrame(animate);
}

// ─── Valid Placement Markers ─────────────────────────────────────────────────

function showValidPlacements() {
    clearMarkers();

    if (!gameState.selectedTile || !gameState.humanMoves) return;

    const tileData = gameState.humanMoves.tiles[gameState.selectedTile];
    if (!tileData) return;

    // Filter placements for current rotation
    const validPlacements = tileData.placements.filter(
        p => p.rotation === gameState.rotation
    );

    for (const placement of validPlacements) {
        const marker = document.createElement("div");
        marker.className = "placement-marker";
        marker.style.left = `${placement.x * TILE_SIZE_PX}px`;
        marker.style.top = `${placement.y * TILE_SIZE_PX}px`;

        // Add preview image
        const previewImg = document.createElement("img");
        previewImg.className = "tile-preview";
        previewImg.src = `${TILE_IMAGES_DIR}/${gameState.selectedTile}.png`;
        previewImg.style.transform = `rotate(${((4 - gameState.rotation) % 4) * 90}deg)`;
        previewImg.draggable = false;
        marker.appendChild(previewImg);

        marker.addEventListener("click", (e) => {
            e.stopPropagation();
            placeTile(placement.x, placement.y);
        });

        dom.boardWrapper.appendChild(marker);
    }
}

function clearMarkers() {
    dom.boardWrapper.querySelectorAll(".placement-marker, .meeple-option").forEach(el => el.remove());
}

// ─── Tile Placement ──────────────────────────────────────────────────────────

function placeTile(x, y) {
    if (gameState.phase !== "placing" || !gameState.selectedTile) return;

    const tileName = gameState.selectedTile;
    const rotation = gameState.rotation;

    // Get meeple options first
    const meepleJson = wasmGame.get_meeple_options(tileName, x, y, rotation);
    const meepleData = JSON.parse(meepleJson);

    // Store pending placement
    gameState.pendingPlacement = { x, y, tileName, rotation };

    // Add the tile visually (without meeple yet)
    const newTile = {
        tile_name: tileName,
        x: x,
        y: y,
        rotation: rotation,
        meeples: [],
    };

    gameState.placedTiles.push(newTile);
    clearMarkers();
    renderBoard();
    scrollToTile(x, y);

    // Animate the placed tile
    const tiles = dom.boardWrapper.querySelectorAll(".board-tile");
    const lastTile = tiles[tiles.length - 1];
    if (lastTile) lastTile.classList.add("just-placed");

    // Show meeple options or finalize
    if (meepleData.options.length > 0 && meepleData.followers_left > 0) {
        gameState.phase = "meeple";
        gameState.meepleOptions = meepleData;
        showMeepleOptions(x, y, meepleData);
        dom.btnSkipMeeple.classList.add("visible");
        setStatus("Place a meeple or skip");
    } else {
        // No meeple options — finalize move directly
        finalizePlacement(tileName, x, y, rotation, -1);
    }
}

function showMeepleOptions(tileX, tileY, meepleData) {
    for (const option of meepleData.options) {
        const el = document.createElement("div");
        el.className = `meeple-option ${option.structure_type}`;
        el.style.left = `${tileX * TILE_SIZE_PX + option.frac_x * TILE_SIZE_PX}px`;
        el.style.top = `${tileY * TILE_SIZE_PX + option.frac_y * TILE_SIZE_PX}px`;
        el.title = option.structure_type;

        el.addEventListener("click", (e) => {
            e.stopPropagation();
            chooseMeeple(option.index);
        });

        dom.boardWrapper.appendChild(el);
    }
}

function chooseMeeple(index) {
    if (gameState.phase !== "meeple" || !gameState.pendingPlacement) return;

    const { tileName, x, y, rotation } = gameState.pendingPlacement;
    finalizePlacement(tileName, x, y, rotation, index);
}

function skipMeeple() {
    if (gameState.phase !== "meeple" || !gameState.pendingPlacement) return;

    const { tileName, x, y, rotation } = gameState.pendingPlacement;
    finalizePlacement(tileName, x, y, rotation, -1);
}

function finalizePlacement(tileName, x, y, rotation, meepleIndex) {
    const resultJson = wasmGame.play_human_move(tileName, x, y, rotation, meepleIndex);
    const result = JSON.parse(resultJson);

    if (!result.success) {
        setStatus(`Error: ${result.error}`);
        // Rollback visual
        gameState.placedTiles.pop();
        renderBoard();
        gameState.phase = "placing";
        showValidPlacements();
        return;
    }

    // If meeple was placed, update the visual record
    if (meepleIndex >= 0 && gameState.meepleOptions) {
        const option = gameState.meepleOptions.options[meepleIndex];
        const lastTile = gameState.placedTiles[gameState.placedTiles.length - 1];
        lastTile.meeples.push({
            structure_type: option.structure_type,
            frac_x: option.frac_x,
            frac_y: option.frac_y,
            player: "human",
        });
    }

    clearMarkers();
    dom.btnSkipMeeple.classList.remove("visible");
    renderBoard();
    updateScores(result.scores);

    if (result.game_over) {
        endGame();
        return;
    }

    // Bot's turn
    startBotTurn();
}

// ─── Tile Grid ───────────────────────────────────────────────────────────────

function renderTileGrid() {
    dom.tileGrid.innerHTML = "";

    // Get fresh tile data from the last computed moves
    const movesData = gameState.humanMoves;

    for (const tileName of ALL_TILE_NAMES) {
        const card = document.createElement("div");
        card.className = "tile-card";
        card.dataset.tileName = tileName;

        const img = document.createElement("img");
        img.src = `${TILE_IMAGES_DIR}/${tileName}.png`;
        img.alt = tileName;
        img.draggable = false;
        card.appendChild(img);

        const tileData = movesData ? movesData.tiles[tileName] : null;
        const count = tileData ? tileData.count : 0;
        const hasPlacement = tileData && tileData.placements.length > 0;

        const countEl = document.createElement("span");
        countEl.className = "tile-count";
        countEl.textContent = `×${count}`;
        card.appendChild(countEl);

        if (count === 0) {
            card.classList.add("unavailable");
        } else if (!hasPlacement) {
            card.classList.add("no-placement");
            card.title = "No valid placement";
        }

        if (count > 0 && hasPlacement) {
            card.addEventListener("click", () => selectTile(tileName));
        }

        dom.tileGrid.appendChild(card);
    }
}

// ─── Scores & UI Updates ─────────────────────────────────────────────────────

function updateScores(scores) {
    if (!scores) return;

    const humanPrev = dom.scoreHumanValue.textContent;
    const botPrev = dom.scoreBotValue.textContent;

    dom.scoreHumanValue.textContent = scores.human;
    dom.scoreBotValue.textContent = scores.bot;
    dom.followersHuman.textContent = `🧍×${scores.human_followers}`;
    dom.followersBot.textContent = `🧍×${scores.bot_followers}`;

    // Flash animation on score change
    if (humanPrev !== String(scores.human)) {
        dom.scoreHumanValue.classList.remove("score-flash");
        void dom.scoreHumanValue.offsetWidth; // trigger reflow
        dom.scoreHumanValue.classList.add("score-flash");
    }
    if (botPrev !== String(scores.bot)) {
        dom.scoreBotValue.classList.remove("score-flash");
        void dom.scoreBotValue.offsetWidth;
        dom.scoreBotValue.classList.add("score-flash");
    }
}

function setStatus(message, highlight = false) {
    dom.statusMessage.textContent = message;
    dom.statusMessage.classList.toggle("highlight", highlight);
}

function formatTileName(name) {
    return name.replace(/_/g, " ");
}

// ─── Board Panning ───────────────────────────────────────────────────────────

function onBoardMouseDown(e) {
    // Only start drag on the board itself, not on markers
    if (e.target.closest(".placement-marker") || e.target.closest(".meeple-option")) return;

    camera.dragging = true;
    camera.startX = e.clientX;
    camera.startY = e.clientY;
    camera.startCamX = camera.x;
    camera.startCamY = camera.y;
}

function onBoardMouseMove(e) {
    if (!camera.dragging) return;

    const dx = e.clientX - camera.startX;
    const dy = e.clientY - camera.startY;
    camera.x = camera.startCamX + dx;
    camera.y = camera.startCamY + dy;
    updateBoardTransform();
}

function onBoardMouseUp() {
    camera.dragging = false;
}

// ─── Start ───────────────────────────────────────────────────────────────────

main().catch(err => {
    console.error("Failed to initialize:", err);
    document.getElementById("loading-overlay").innerHTML =
        `<span class="loading-text" style="color: #f85149;">Error loading game. Check console.</span>`;
});
