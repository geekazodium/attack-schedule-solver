extends Node2D

var velocity: Vector2;

func _ready() -> void:
	self.velocity =  Vector2.from_angle(randf_range(PI, -PI));
	self.velocity.y = -abs(self.velocity.y);
	self.velocity *= 1000;

func _on_timer_timeout() -> void:
	self.queue_free();

func _process(delta: float) -> void:
	self.velocity += Vector2.DOWN * 1000 * delta;
	self.position += self.velocity * delta;
