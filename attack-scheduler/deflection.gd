extends Node2D

@export var buttons: Dictionary[StringName, ShapeCast2D];

@export var active_timer: Timer;
@export var cooldown_timer: Timer;

signal active_end();

func _ready() -> void:
	self.active_timer.timeout.connect(self.active_end.emit);

func _physics_process(delta: float) -> void:
	for key in buttons.keys():
		if Input.is_action_just_pressed(key):
			self.try_parry(key);

func try_parry(key: StringName) -> void:
	if !self.cooldown_timer.is_stopped():
		return;
	self.enable(self.buttons[key]);
	self.cooldown_timer.start();
	self.active_timer.start();

func on_area_entered(area: Area2D) -> void:
	if !self.active_timer.is_stopped():
		self.stop_active();
		self.cooldown_timer.stop();
		area.queue_free();

func stop_active() -> void:
	self.active_timer.stop();
	self.active_end.emit();

func enable(shapecast: HitShapeCast) -> void:
	shapecast.area_entered.connect(self.on_area_entered);
	shapecast.visible = true;
	shapecast.enabled = true;
	await self.active_end;
	shapecast.area_entered.disconnect(self.on_area_entered);
	shapecast.visible = false;
	shapecast.enabled = false;
