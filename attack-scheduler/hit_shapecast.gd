extends ShapeCast2D
class_name HitShapeCast

signal area_entered(area: Area2D);

func _physics_process(_delta: float) -> void:
	self.force_shapecast_update();
	if self.is_colliding():
		self.area_entered.emit(self.get_collider(0));
