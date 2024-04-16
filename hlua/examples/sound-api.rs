#[macro_use]
extern crate hlua;

fn main() {
    let mut lua = hlua::Lua::new();
    lua.openlibs();

    // we create a fill an array named `Sound` which will be used as a class-like interface
    {
        let mut sound_namespace = lua.empty_array("Sound");

        // creating the `Sound.new` function
        sound_namespace.set("new", hlua::function0(|| Sound::new()));
    }

    lua.execute::<()>(
        r#"
        s = Sound.new();
        s:play();

        print("hello world from within lua!");
        print("is the sound playing:", s:is_playing());

        s:stop();
        print("is the sound playing:", s:is_playing());

    "#,
    )
    .unwrap();
}

// this `Sound` struct is the object that we will use to demonstrate hlua
struct Sound {
    playing: bool,
}

// this macro implements the required trait so that we can *push* the object to lua
// (ie. move it inside lua)
implement_lua_push!(Sound, |mut metatable| {
    // we create a `__index` entry in the metatable
    // when the lua code calls `sound:play()`, it will look for `play` in there
    let mut index = metatable.empty_array("__index");

    index.set("play", hlua::function1(|snd: &mut Sound| snd.play()));

    index.set("stop", hlua::function1(|snd: &mut Sound| snd.stop()));

    index.set(
        "is_playing",
        hlua::function1(|snd: &Sound| snd.is_playing()),
    );
});

// this macro implements the require traits so that we can *read* the object back
implement_lua_read!(Sound);

impl Sound {
    pub fn new() -> Sound {
        Sound { playing: false }
    }

    pub fn play(&mut self) {
        println!("playing");
        self.playing = true;
    }

    pub fn stop(&mut self) {
        println!("stopping");
        self.playing = false;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }
}

// this destructor is here to show you that objects are properly getting destroyed
impl Drop for Sound {
    fn drop(&mut self) {
        println!("`Sound` object destroyed");
    }
}
