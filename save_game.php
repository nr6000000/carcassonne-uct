<?php
// Set headers to return JSON response
header('Content-Type: application/json');

// 1. Database configuration (Update these with your credentials)
$host     = 'localhost';
$db       = 'your_database_name';
$user     = 'your_username';
$password = 'your_password';

try {
    // Connect to MySQL using PDO
    $pdo = new PDO("mysql:host=$host;dbname=$db;charset=utf8mb4", $user, $password, [
        PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
        PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC
    ]);

    // 2. Get the raw JSON payload from the JS fetch request
    $json = file_get_contents('php://input');
    $data = json_decode($json, true);

    // Validate that we actually got data
    if (!$data || !isset($data['player_name'], $data['engine_name'], $data['player_score'], $data['opponent_score'])) {
        echo json_encode(['success' => false, 'error' => 'Invalid input data.']);
        exit;
    }

    // 3. Prepare the SQL statement (Notice we skip 'id' and 'winner' entirely)
    $sql = "INSERT INTO player_matches (player_name, engine_name, player_score, opponent_score) 
            VALUES (:player_name, :engine_name, :player_score, :opponent_score)";
            
    $stmt = $pdo->prepare($sql);

    // 4. Execute with the data
    $stmt->execute([
        ':player_name'    => $data['player_name'],
        ':engine_name'    => $data['engine_name'],
        ':player_score'   => (int)$data['player_score'],
        ':opponent_score' => (int)$data['opponent_score']
    ]);

    // Send a success response back to JS
    echo json_encode(['success' => true, 'message' => 'Match recorded!']);

} catch (PDOException $e) {
    // Handle database connection or query errors
    echo json_encode(['success' => false, 'error' => 'Database error: ' . $e->getMessage()]);
}
?>