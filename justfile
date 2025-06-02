dev-wasm:
    RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo run --target wasm32-unknown-unknown -- 

build-release-wasm:
    ./bash-scripts/build-release-wasm.sh

astro-dev:
    cd astro && astro dev

# Wurzeln generieren und konvertieren
roots:
    python3 scripts/generate_tree_root_particles.py
    blender --background --python scripts/root_obj_to_gltf.py

# Bäume generieren und konvertieren
trees:
    python3 scripts/generate_trees.py
    blender --background --python scripts/batch_trees_obj_to_gltf.py
