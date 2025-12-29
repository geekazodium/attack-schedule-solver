extends Node

@export var track: MovesTrack;

@export var next_request_timer: Timer;

func _physics_process(_delta: float) -> void:
	if next_request_timer.is_stopped():
		track.attack_track.commit_move_now(0);
		next_request_timer.start();
