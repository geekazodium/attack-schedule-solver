extends Node2D

@export var attack_track: ExternEnemyTrack;

@export var placeholder_spawn_scene: PackedScene;

var current_move: ExternEnemyAttack;
var current_move_frame: int = 0;

signal active_frame_start();

func instantiate_test_with_color(color: Color) -> void:
	var scene = self.placeholder_spawn_scene.instantiate();
	if scene is Node2D:
		scene.modulate = color;
	self.add_child(scene);

func _physics_process(_delta: float) -> void:
	var updated: bool = self.try_update_current_move();
	var index = attack_track.attack_index_on_this_frame();
	if index < 0:
		return;
	if updated:
		push_warning("attempting to start new move while last move was still active");
	print(self.get_instance_id(), ", doing move index: ", index);
	self.start_attack(index);

func start_attack(index: int) -> void:
	self.current_move = self.attack_track.attacks[index];
	self.current_move_frame = 0;

func try_update_current_move() -> bool:
	if self.current_move != null:
		if self.current_move_frame > self.current_move.duration:
			self.current_move = null;
	if self.current_move == null:
		return false;
	self.update_current_move();
	self.current_move_frame += 1;
	return true;

func update_current_move() -> void:
	# possibly slightly inefficient, could just use a counter instead for o(1)
	var index = self.current_move.frames.bsearch(self.current_move_frame);
	if self.current_move.frames.size() <= index:
		return;
	if self.current_move.frames[index] != self.current_move_frame:
		return;
	self.active_frame_start.emit();
	var scene = self.placeholder_spawn_scene.instantiate();
	self.add_child(scene);
