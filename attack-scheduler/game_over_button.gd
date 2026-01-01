extends Button

@export var tracks: Array[Node];

func _physics_process(_delta: float) -> void:
	if tracks.all(self._track_depleted):
		self.visible = true;

func _pressed() -> void:
	self.get_tree().reload_current_scene();

func _track_depleted(track) -> bool:
	return track.depleted
