extends Node2D

@export var fall_speed: float = 300;

func _physics_process(delta: float) -> void:
	self.position += Vector2.DOWN * delta * fall_speed;
	if self.position.y - $Sprite2D.get_rect().size.y * $Sprite2D.scale.y / 2 > 800:
		self.queue_free();
