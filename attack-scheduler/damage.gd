extends Area2D

@export var max_health: int;
var current_health: int;

func _ready() -> void:
	self.area_entered.connect(self.on_area_entered);
	self.current_health = self.max_health;
	$ProgressBar.max_value = self.max_health;
	$ProgressBar.value = self.current_health;

func on_area_entered(_area: Area2D) -> void:
	self.current_health -= 1;
	$ProgressBar.value = self.current_health;
	if self.current_health <= 0:
		self.get_tree().reload_current_scene.call_deferred();
