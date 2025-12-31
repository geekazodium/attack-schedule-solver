extends Node2D
class_name MovesTrack;

const parry_dir_color: Array[Color] = [Color.RED, Color.BLUE];
const parry_collision_layer: Array[int] = [0b01, 0b10];

@export var attack_track: ExternEnemyTrack;

@export var placeholder_spawn_scene: PackedScene;

var current_move: ParryableEnemyAttack;
var current_move_frame: int = 0;

signal active_frame_start();

func _enter_tree() -> void:
	self.attack_track = self.attack_track.duplicate(false);
	GlobalSolverInterface.add_track(self.attack_track);

func _exit_tree() -> void:
	GlobalSolverInterface.remove_track(self.attack_track);

func instantiate_no_collide(color: Color) -> Node2D:
	var v = instantiate_with_color(color);
	v.collision_layer = 0;
	return v;

func instantiate_with_color(color: Color) -> Node2D:
	var scene = self.placeholder_spawn_scene.instantiate();
	if scene is Node2D:
		scene.modulate = color;
	self.add_child(scene);
	return scene;

func instantiate_parry(parry_dir: int) -> Area2D:
	var scene: Area2D = self.instantiate_with_color(self.parry_dir_color[parry_dir]);
	scene.collision_layer = self.parry_collision_layer[parry_dir];
	return scene;

func instantiate_no_collide_with_height(length: int):
	var instance = self.instantiate_no_collide(Color(.2,.2,.2,.2));
	var sprite: Sprite2D = instance.find_child("Sprite2D");
	var size: float = sprite.get_rect().size.y;
	var target_y_size = instance.fall_speed * length / Engine.physics_ticks_per_second;
	sprite.scale.y = target_y_size / size;
	instance.position.y -= sprite.get_rect().size.y * sprite.scale.y / 2;
	instance.z_index = -10;

func _physics_process(_delta: float) -> void:
	var index = attack_track.attack_index_active_now();
	var start_index = attack_track.attack_index_on_this_frame();
	var updated: bool = self.try_update_current_move();
	if index < 0:
		#self.validate();
		return;
	if start_index >= 0:
		self.start_attack(start_index);
		if updated:
			push_warning("attempting to start new move while last move was still active");
	if self.current_move_frame != attack_track.attack_frame_active_now():
		print(self.current_move_frame - attack_track.attack_frame_active_now())
		push_error("desync! please fix me.");

func validate() -> void:
	var active_now = attack_track.attack_index_active_now();
	var expected_current_move;
	if active_now < 0:
		expected_current_move = attack_track.attacks[active_now];
	else:
		expected_current_move = null;
	if expected_current_move != self.current_move:
		push_error("what")
		print(self.current_move_frame - attack_track.attack_frame_active_now())

func start_attack(index: int) -> void:
	self.current_move = self.attack_track.attacks[index];
	self.instantiate_no_collide_with_height(self.current_move.duration);
	self.current_move_frame = 0;

func try_update_current_move() -> bool:
	if self.current_move != null:
		if self.current_move_frame > self.current_move.duration:
			self.current_move = null;
	if self.current_move == null:
		self.current_move_frame = -1;
		return false;
	self.update_current_move();
	self.current_move_frame += 1;
	return true;

func update_current_move() -> void:
	# possibly slightly inefficient, could just use a counter instead for o(1)
	var index = self.current_move.frames.bsearch(self.current_move_frame);
	if self.current_move.frames.size() > index:
		if self.current_move.frames[index] == self.current_move_frame:
			self.active_frame_start.emit();
			var parry_dir: int = self.current_move.parry_directions[index];
			self.instantiate_parry(parry_dir);
