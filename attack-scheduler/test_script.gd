extends Node

@export var track: MovesTrack;

func _physics_process(_delta: float) -> void:
	track.attack_track.commit_move_now(0);
