extends Node2D

@export var attack_track: ExternEnemyTrack;

func _physics_process(_delta: float) -> void:
	attack_track.commit_move_now(0);
