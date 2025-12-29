extends Node2D

@export var fall_speed: float = 300;

func _physics_process(delta: float) -> void:
	self.position += Vector2.DOWN * delta * fall_speed;
	if self.position.y > 1000:
		self.queue_free();
