extends ProgressBar

signal dead();

func _ready() -> void:
	self.value = self.max_value;

func _track_parry_instantiated(parry: Node) -> void:
	parry.parried.connect(self.on_parry);

func on_parry() -> void:
	self.value -= 1;
	if self.value <= 0:
		self.dead.emit();
