extends Node2D

@export var fall_speed: float = 300;

var _parried : bool = true;

signal parried();

func _ready() -> void:
	self.tree_exiting.connect(self.on_tree_exiting)

func _physics_process(delta: float) -> void:
	self.position += Vector2.DOWN * delta * fall_speed;
	if self.position.y - $Sprite2D.get_rect().size.y * $Sprite2D.scale.y / 2 > 800:
		self.queue_free();
		self._parried = false;

func on_tree_exiting() -> void:
	if self._parried:
		parried.emit();
