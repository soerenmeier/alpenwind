let npmPackages = [
	"core/ui",
	"core-lib/js",
	"core-lib/ui",
	"cinema/ui",
	"pwvault/ui"
];

let apps = [
	"core",
	"cinema",
	"pwvault"
];

fn help() {
	print([
		"- npm_install",
		"- build <app>",
		"- build_core_lib"
	]);
}

fn npm_install() {
	for pack in npmPackages {
		let npm = cmd(["npm", "install"]);
		npm.dir("./" + pack);
		npm.execute();
	}
}

fn npm_ci() {
	for pack in npmPackages {
		let npm = cmd(["npm", "ci"]);
		npm.dir("./" + pack);
		npm.execute();
	}
}

fn npm_update() {
	for pack in npmPackages {
		let npm = cmd(["npm", "update"]);
		npm.dir("./" + pack);
		npm.execute();
	}
}

fn cargo_update() {
	for app in apps {
		let cargo = cmd(["cargo", "update"]);
		cargo.dir("./" + app + "/server");
		cargo.execute();
	}

	let cargo = cmd(["cargo", "update"]);
	cargo.dir("./core-lib/build");
	cargo.execute();

	let cargo = cmd(["cargo", "update"]);
	cargo.dir("./core-lib/lib");
	cargo.execute();
}

// how does the build process work

/// config should be
/// [apps]
/// dir = "./apps"
fn build_all() {
	for app in apps {
		build(app);
	}

	fs::delete("./dist");
	fs::create_dir("./dist");

	fs::copy("./core/server/target/release/core-server", "./dist");

	fs::create_dir("./dist/apps");

	for app in apps {
		if app == "core" {
			continue
		}

		let app_dir = "./dist/apps/" + app;
		fs::create_dir(app_dir);
		fs::copy(
			"./" + app + "/server/target/release/lib" + app + "_server.so",
			app_dir + "/" + app + ".so"
		);
	}
}

fn build(app) {
	let npm = cmd(["npm", "run", "build"]);
	npm.dir("./" + app + "/ui");
	npm.execute();

	if app == "core" {
		build_core_lib();
	}

	let cargo = cmd(["cargo", "build", "--release"]);
	cargo.dir("./" + app + "/server");
	cargo.execute();
}

fn build_core_lib() {
	let npm = cmd(["npm", "run", "build"]);
	npm.dir("./core-lib/js");
	npm.execute();
}

/// build everything to see if it works
fn test_build() {
	for app in apps {
		test_build(app);
	}
}

fn test_build(app) {
	let npm = cmd(["npm", "run", "build"]);
	npm.dir("./" + app + "/ui");
	npm.execute();

	if app == "core" {
		build_core_lib();
	}

	let cargo = cmd(["cargo", "build"]);
	cargo.dir("./" + app + "/server");
	cargo.execute();
}

fn build_docker() {
	fs::create_dir("./images");

	let build = cmd([
		"docker", "build", "-t", "alpenwind", "."
	]);
	build.execute();

	let save = cmd([
		"docker", "save", "-o", "./images/alpenwind.tar", "alpenwind"
	]);
	save.execute();
}