extends Node

@export var tracks: Array[MovesTrack];

@export var next_request_timer: Timer;

func _physics_process(_delta: float) -> void:
	if next_request_timer.is_stopped():
		var track = self.tracks[randi_range(0,self.tracks.size() - 1)].attack_track;
		track.commit_move_now(randi_range(0,track.attacks.size() - 1));
		next_request_timer.start();
