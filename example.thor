let mjolnir = import_lib("mjolnir.so");

let window = mjolnir.create_window();

let square_side = 200;
let square_position = [0, 0];
let square_speed = 20;

//make pen draw in white
window.set_color([255, 255, 255]);

while(true){

	let delta = window.get_delta_time();

	//set target frame rate to 16 (60 fps)
	let target = 16;

	//we update the square speed with the delta time so inconsisten framrates lead to consistent speeds
	square_position[0] = square_position[0] + square_speed * delta * 10;

	//in reality square_position[0] can be much bigger than the screens width, if that is the case,
	//the x coordinates will sort of "wrap around" (**but increase by 1 in the y coordinate!**)


	window.draw_rect([
		square_position, 
		[square_position[0] + square_side, square_position[1] + square_side]
	]);

	print delta;

	if(delta < target){
		//if the remaining time each frame is smaller than the target framerate we wait
		window.sleep(target - delta);
	}
	

	//request a new frame and delete all the drawn stuff
	window.flush();
}
