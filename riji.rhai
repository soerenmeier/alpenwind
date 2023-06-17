let apps = ["core/ui", "core-lib/js", "core-lib/ui", "cinema/ui", "pwvault/ui"];

fn npm_install() {
	for app in apps {
		let npm = cmd(["npm", "install"]);
		npm.dir("./" + app);
		npm.execute();
	}
}

// how does the build process work

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