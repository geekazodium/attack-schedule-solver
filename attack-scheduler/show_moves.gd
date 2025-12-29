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
	self.instantiate_no_collide(Color(.2,.2,.2,.2));
	self.current_move_frame = 0;

func try_update_current_move() -> bool:
	if self.current_move != null:
		if self.current_move_frame > self.current_move.duration:
			self.instantiate_no_collide(Color(.7,.0,.7,.2));
			self.current_move = null;
	if self.current_move == null:
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
	index = self.current_move.requests.bsearch(self.current_move_frame);
	if self.current_move.requests.size() > index:
		if self.current_move.requests[index] == self.current_move_frame:
			self.instantiate_no_collide(Color(.2,1,.2,.3)).scale.x *= 100;
