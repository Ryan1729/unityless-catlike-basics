# Unityless Catlike Basics

I ran across [Catlike Coding's Unity tutorials](https://catlikecoding.com/unity/tutorials) and found them to be very well done. So I wanted to give following them a try. The one problem is I prefer to not spend my time building things on platforms that might go away, so I'm not particularly interested in learning Unity specifically. From my perspective, Unity seems to be in a more precarious position than building on Open Source software. (As of this writing, Unity has some C# code which is viewable for reference purposes only, so "source avaialble", but not Open Source.)

So, instead of directly following the tutorials, I'm attempting to recreate the output without the Unity engine. I will likely be using some sort of other third party engine/libraries however. At this time I would prefer to write the bulk of the code in Rust, but I'm not promising anything in that regard.

I'm not planning on putting an excessive amount of effort into making things look exactly the same, but I will try to keep the fidelity high when the effort involved is reasonable. For example, there's a skybox by default in Unity, and there's skybox examples for many engines/libraries, so I might as well include one. It's pretty easy to make a skybox texture that resembles the Unity default one, but I'm not worrying about making it look identical.

As another example of how much fidelity I'm aiming for, when representing Unity prefabs, I will use a `struct` and create multiple instances of it, since that gets across the general feel of how the code works. I will not be worrying about replicating Unity's inheritance hierarchy in any way whatsoever, since I don't think that would be worthwhile. Regarding the GPU, whether something happens on the GPU or not is often an important performance difference, in practice. So, I will attempt to put things that happen on the GPU inthe tutorials, onto the GPU in my version, when it is feasible with a reasonalbe amount of effort. I do not expect to write to end up writing some kind of uber shader, or compiler that outputs shader code, for example.

I'm hoping doing this will cause me to both absorb the engine-agnostic content of the tutorials, and semi-incidentally teach some things that I wouldn't have learned by just following the tutorials directly.

----

The third-party libraries/engines are under their own licenses. See the separate license files and/or the files themselves for details.

Everything else licensed under MIT-0, same as the original tutorials.