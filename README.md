
# Shipyard Relations

This crate provides Entity Relations for [Shipyard](https://github.com/leudz/shipyard), an awesome ECS (Entity Component System) library for Rust. Internally it uses a graph structure provided by [petgraph](https://github.com/petgraph/petgraph).

## Motivation

Relations between entities are very common in games. For example, a monster entity seeks a target entity. A container entity contains an item entity.

One way to relate an entity to another is simply to store the `EntityId` of one inside a component of the other.

This can lead to problems, if you don't check the entity's aliveness. For example a hierarchy that relies on components storing related entity ids might break when entities are deleted (and this case isn't properly handled).

`shipyard_relations` tries to make relations between entities safer and more convenient.

## Features

- Deleting one of the two entities in the relation also deletes the whole relation.
- Every relation can come with additional data.
- Different kinds of relations are supported:
   - `Directed`
   - `DirectedExclusive`
   - `DirectedExclusiveIncoming`
   - `DirectedExclusiveOutgoing`
   - `Undirected`
   - `UndirectedExclusive`
- Can detect or prevent cycles.
- Tracks insertions and deletions of relations (so you can react to them).


## Usage

To define a Relation, simply implement the `Relation` trait for a struct with or without data.

```rust
use shipyard_relations::Relation;

struct Friends;

impl Relation for Friends {
    type Mode = Undirected;
}
```

Let's add entities and make them friends: 

```rust
use shipyard::World;
use shipyard_relation::RelationExt;

let mut world = World::new();

let a = world.add_entity(());
let b = world.add_entity(());
let c = world.add_entity(());

world.add_relation(a, b, Friends).unwrap();
world.add_relation(a, c, Friends).unwrap();
```

Note that adding the relation `(a, b, Friends)` is the same as `(b, a, Friends)` since we chose the `Undirected` mode. Had we chosen a directional mode, this would be a distinct relation.

Now `a, b` are friends as well as `a, c`. With `UndirectedExclusive`, adding `a, c` would replace `a, b` (exclusiveness meaning that only one relation of this kind can exist for an entity).

To access relation inside systems for example, you simply borrow `RelationView` or `RelationViewMut`:

```rust
fn system(v_person: View<Person>, r_friends: RelationView<Friends>) {
  for (id, _) in v_person.iter().with_id() {
    for other_id in r_friends.get(id) {
      // do stuff
    }
  }
}
```

Here we get an iterator when calling `get`, because an entity can have multiple `Friends` relations. With an *exclusive* undirected relation, we'd get an `Option` instead, because then there can only exist one at maximum.



## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.