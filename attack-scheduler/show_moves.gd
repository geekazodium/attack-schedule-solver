extends Node

@export var attack_track: ExternEnemyTrack;

func _physics_process(delta: float) -> void:
	var index = attack_track.attack_index_on_this_frame();
	if index < 0:
		return;
	print(self.get_instance_id(), ", doing move index: ", index);
