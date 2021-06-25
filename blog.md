# Audio Scheduling
Code examples are in typescript, not because you should write an audio engine in typescript but because it is a convenient language for proto**typing**. My own experience with TS is limited, so please forgive any unorthodoxies.

--- 
## Problem Definition
### What is "audio scheduling"? 

Scheduling is a core task of any node-based workflow. It is the process of ordering the processes associated with nodes, allocating resources for them, and compiling a data representation that can be consumed to efficiently render the audio output. 

### What is a "node" ? 

A "node" is a subroutine that takes some input and writes some output. We graphically represent it as a list of input ports and output ports. In programming terms, the "input ports" are arguments to the subroutine and the "output ports" are its return values. The corrolary to compilers and programming languages will not go away the further you read. 

To make our lives easier, we're going to assume that input and output data may only be one type: the humble `Buffer`. We don't care what it looks like internally.

```ts
type Buffer = {
    /* ... */
}
```

A `Node` will have some `process` and some `Port`s. The `Port`s define the input and outputs of the node, and `process` is a function that takes the input and output `Buffer`s assigned to each `Port`. We call the assigned buffers a `BufferAssignment`, which is analogous to a value bound to an argument in a function call. 

`Port`s have an optional `connection` to another node - for input `Port`s, the `connection` is a corresponding output and for an output `Port` is a corresponding input. This can be thought of as a kind of function composition, except `Node`s may be composed in parallel and all inputs and outputs are optional.

```ts
type BufferAssignment = {
    port:   string, 
    buffer: Buffer,
}

type Port = {
    name:string,
    connection?:{ node:Node, port:string}
}

type Node = {
    process: (inputs:BufferAssignment[], outputs:BufferAssignment[]) => void,
    inputs: Port[]
    outputs: Port[]
}
```

### Rendering the Graph

To finally render the audio, we need to collect this information into a list of nodes and assigned buffers so that we can render the audio. We call this assignment a "scheduled" node, and the list of all scheduled nodes the "schedule." Rendering is then a simple iteration over the schedule.

```ts
type Scheduled = {
    node:    Node,
    inputs:  BufferAssignment[],
    outputs: BufferAssignment[]
}

function render (schedule:Scheduled[]) {
    for (let { node, inputs, outputs } of schedule) {
        node.process(inputs, outputs)
    }
}
```

### Compiling the Schedule

The problem of audio scheduling is taking the input data representation, a `Graph`, and converting it into the output data representation that can be consumed by the `render` function, `Scheduled[]`. 

---

## Simple Scheduling

The easiest thing we can do is topologically sort the graph using depth-first traversal. This is easy with just a little extra bookkeeping in our definition of `Node`: 

```ts
type Node = {
    /* ... */
    visited = boolean
}
```

Now we can walk the graph topologically, collecting the order of nodes along the way.

```ts
function schedule (root:Node): Node[] {
    let order = []
    function visit (node:Node) {
        if (node.visited) {
            return
        }
        for (let next of node.inputs) { 
            visit(next.node)
        }
        // add to the order and mark the node as visited
        order.push(node)
        node.visited = true
    }
    visit(root)
    return order
}
```

We still need to figure out the buffer assignments. An easy way to do this is to assign a buffer for every unique edge in the graph. We can add a little extra bookkeeping to `Port`s 

```ts
type Port = {
    ... 
    buffer?:Buffer // each port must keep track of its buffer assignment.
}

function schedule (root:Node): Scheduled[] {
    let order = []

    function visit (node:Node): BufferAssignments[] {
        if (!node.visited) {
            // for each output in this port, create a new buffer.
            for (let output in node.outputs) {
                output.buffer = new Buffer
            }

            // for each input port, find an input buffer
            for (let input in node.inputs) {
                // if the input port is connected, solve the node
                // on the other side and find the corresponding output port. 
                if (input.connection) {
                    let outputs = visit(input.connection.node as Node)
                    let correspondingOutput = outputs.find(asgn => asgn.port === input.connection.port)
                    input.buffer = corresondingOutput.buffer
                } else {
                    // if there is no connection, assign a new buffer.
                    input.buffer = new Buffer
                }
            }

            // collect the input and output buffers
            let inputs  = node.inputs.map(port => { port: port.name, buffer: port.buffer as Buffer})
            let outputs = node.outputs.map(port => { port: port.name, buffer: port.buffer as Buffer})
            
            // add this node to the input/output assignments
            order.push({ node, inputs, outputs })

            // mark the node as visited.
            node.visited = true
        }
        
        // return the output buffer assignments, relative to their corresponding input port.
        return node.outputs.map((port) => {
            port:   port.connection ? port.connection.port : "",
            buffer: port.buffer as Buffer
        })
    }

    visit(root)
    return order
}
```

## An Improved Scheduler

One of the undesirable effects of this algorithm is that it creates new buffers for every edge. Once a node has been processed, and all its outputs have been handled by corresponding inputs, we can reuse the same memory. 

Continuing the parallel to compilers - this is almost identical to a register allocation problem. The advantages we have are that we don't have a fixed number of registers or care about spillage, and all ~~buffers~~ registers are the same type.

A quick and dirty way to do this is to use a stack of previously used buffers. When we need a buffer, we "acquire" one by checking if one is available in the stack - if not, create a new one. When we're done, we "release" the buffer by placing it on the top of the stack. A queue may also be used in place of the stack.

```ts
function schedule (root:Node): Scheduled[] {
    let order = []
    // the "buffer stack" is used to track buffers we plan to reuse.
    let bufferStack = [] 
    
    // "acquiring" a buffer means to create a new buffer if none are available, otherwise
    // pop from the stack
    function acquireBuffer(): Buffer {
        let buffer = bufferStack.pop()
        return buffer ? buffer : {} // create a new buffer if it does not exist
    }

    // "releasing" a buffer pushes it to the top of the stack
    function releaseBuffer(buffer:Buffer) {
        bufferStack.push(buffer)
    }

    function visit (node:Node): BufferAssignments[] {
        if (!node.visited) {
            // for each output in this port, acquire a new buffer.
            for (let output in node.outputs) {
                output.buffer = acquireBuffer() // << acquire, rather than allocate a buffer.
            }

            // for each input port, find the input buffer
            for (let input in node.inputs) {
                // if the input port is connected, solve the node on the other side 
                // and find the corresponding output buffer
                if (input.connection) {
                    let outputs = visit(input.connection.node)
                    let correspondingOutput = outputs.find(
                        (asgn) => asgn.port === input.connection.port
                    )
                    input.buffer = corresondingOutput.buffer
                } else {
                    // if there is no connection, acquire a new buffer.
                    input.buffer = acquireBuffer()
                    // since the buffer is not going to be reused, we can release it immediately.
                    releaseBuffer(input.buffer)
                }
            }

            // remember to release the buffers!
            for (let output in node.outputs) {
                releaseBuffer(output.buffer)
            }

            // collect the input and output buffers
            let inputs  = node.inputs.map(port => { port: port.name, buffer: port.buffer as Buffer})
            let outputs = node.outputs.map(port => { port: port.name, buffer: port.buffer as Buffer})
            
            // add this node to the input/output assignments
            order.push({ node, inputs, outputs })

            // mark the node as visited.
            node.visited = true
        }
        
        // return the output buffer assignments, relative to their corresponding input port.
        return node.outputs.map((port) => {
            port:   port.connection ? port.connection.port : "",
            buffer: port.buffer as Buffer
        })
    }

    visit(root)
    return order
}
```

## The Invariants

Before continuing it's worth mentioning the invariants explicitly. These invariants are properties of the graph that must be upheld throughout the execution of the algorithm. 

### 1. Acyclicity 

We've already presented one invariant to our representation - the graph must not contain a cycle. This is required for topological ordering ; a topological ordering does not exist in a graph that contains a cycle. 

A cycle in an audio graph is known in the business as "feedback" and it's not generally desirable... but paradoxically, something one could reasonably assume is undesirable on an electrical or controls-theoretical basis often winds up to be extraordinarily desirable by the professional audio community. 

Breaking cycles is outside the scope of this algorithm. While not tested, the author believes that the algorithm above will handle cycles without crashing or spinning in an infinite loop, but the delay around the cycle will be undefined.

### 2. One-to-Many Connections

It has been implicit so far, but the second invariant is that the algorithm must only allow one-to-many connections, while many-to-one are unhandled. This is implied by the portion of the algorithm that finds the input buffer assignments. This can be solved by defining special-purpose "merge" nodes of the graph and inserting them in a pre-processing step (or perhaps, during construction/mutation of the graph).

### 3. Graph Stability

This algorithm requires the graph to be _stable_. What that means is edges must not be inserted or removed during the traversal. For example, the one-to-many invariant may be avoided by inserting nodes _during_ the scheduling pass - the reason this breaks down is that to solve a node, we must solve all of its input nodes first. Inserting or removing edges from the graph may invalidate the partial solution prior to completion, which manifests as an unsound buffer assignment (you may overwrite buffers that should be preserved during rendering).

> Note to self: prove this or find a degenerate case rather than assert from experience with a prior iteration of the algorithm that was slightly different

### 4. There exists a root node

The "root" of a graph is a node with out degree zero that is reachable from all nodes in the graph. In an audio system, this is the final audio output. 

In the case that there is no root, or not all nodes are connected to it, a "pseudo root" may be defined by creating a node with input edges from every node with outdegree zero.

---

## The Elephant in the Room is Late to the Party

It would seem the algorithm is complete, and all cases are handled. We can render audio!

There is a small hiccup in the nature of _what_ audio processing nodes do to their inputs before writing their outputs. It can be incredibly advantageous to allow these nodes to _delay_ their inputs before writing their output. 

This delay is baked into many algorithms, from dynamics processors (lookahead) to convolution, realtime processing in the frequency domain, and filtering. 

If non-zero delay exists within some nodes, or varies across nodes, then the audio signals that pass along different paths of the graph will no longer be time aligned. Most nodes assume their inputs will be time aligned - the result of the discrepency is phase interference and other audible artifacts that exist solely due to the topology of our system!

The scheduling algorithm can fix this - with additional bookkeeping, of course.

## Latency Compensation

We start with definitions:

- The `delay` of a `Node` is the inherent time delay imparted to all information that flows through `Node`, from input to outut. 
- the `latency` of a `Port` is how much time it takes data to reach that port.
- The `latency` of a `Node` is the maximum latency of all its `inputs`, plus its `delay`. 
- `latency` is undefined for output ports.

We add these fields to our `Node` and `Port` definitions

```ts
type Port = {
    ...
    latency?: number
    compensation?: number
}

type Node = {
    delay: number,
    latency?: number
}
```

> Note: naming convention is weird. Should it be called "latency" of a port, or "time-of-arrival"? And the "latency" of a Node, is it well-defined as a reader might expect?

We can solve for the latency of any `Node` recursively. 

```ts
function solveLatency (root:Node): void {
    function visit (node:Node): number {
        if (!node.visited) {
            // first find the latency of all the input ports to this node.
            for (let inputPort of node.inputs) {
                if (inputPort.connection) {
                    inputPort.latency = solveLatency(inputPort.connection.node)
                } else {
                    inputPort.latency = 0
                }
            }
            // now find the maximum of our input latencies
            let maxInputLatency = node.inputs
                .map(port => port.latency as number)
                .reduce((prev, curr) => Math.max(prev, curr), 0)
            
            // compute the compensation required at each port
            for (let inputPort of node.inputs) {
                let compensation = maxInputLatency - inputPort.latency
                if (compensation != 0) {
                    inputPort.compensation = compensation
                }
            } 

            // finally, compute the latency of this node.
            node.latency = maxInputLatency + node.delay
            
            // mark the node as visited
            node.visited = true
        }
        return node.latency as number
    }
}
```

Since this is a depth first traversal like our algorithm above, we can make it a part of the scheduling algorithm to solve for latency compensations in the same traversal of the graph. 

```ts
function schedule (root:Node): Scheduled[] {
    let order = []
    // the "buffer stack" is used to track buffers we plan to reuse.
    let bufferStack = [] 
    
    // "acquiring" a buffer means to create a new buffer if none are available, otherwise
    // pop from the stack
    function acquireBuffer(): Buffer {
        let buffer = bufferStack.pop()
        return buffer ? buffer : {} // create a new buffer if it does not exist
    }

    // "releasing" a buffer pushes it to the top of the stack
    function releaseBuffer(buffer:Buffer) {
        bufferStack.push(buffer)
    }

    // the visit function returns the latency of the node and its output buffer assignments
    function visit (node:Node): { latency: number; outputs: BufferAssignment[] }{
        if (!node.visited) {
            // for each output in this port, acquire a new buffer.
            for (let output in node.outputs) {
                output.buffer = acquireBuffer() // << acquire, rather than allocate a buffer.
            }

            // for each input port, find the input buffer
            for (let input in node.inputs) {
                // if the input port is connected, solve the node on the other side 
                // and find the corresponding output buffer
                if (input.connection) {
                    let { latency, outputs } = visit(input.connection.node)
                    let correspondingOutput = outputs.find(
                        (asgn) => asgn.port === input.connection.port
                    )
                    input.latency = latency
                    input.buffer = corresondingOutput.buffer
                } else {
                    // if there is no connection, acquire a new buffer and set the latency.
                    input.latency = 0
                    input.buffer = acquireBuffer()
                    // since the buffer is not going to be reused, we can release it immediately.
                    releaseBuffer(input.buffer)
                }
            }

            // remember to release the buffers!
            for (let output in node.outputs) {
                releaseBuffer(output.buffer)
            }

            // compute the max latency 
            let maxInputLatency = node.inputs
                .map((port) => port.latency as number)
                .reduce((prev, curr) => Math.max(prev, curr), 0)

            // compute the input compensations
            for (let input of node.inputs) {
                let compensation = maxInputLatency - (input.latency as number)
                if (compensation != 0) {
                    input.compensation = compensation
                }
            }

            // update the latency of this node 
            node.latency = maxInputLatency + node.delay

            // collect the input and output buffers
            let inputs  = node.inputs.map((port) => {
                return {
                    port: port.name, 
                    buffer: port.buffer as Buffer,
                    compensation: port.compensation // compensation is now a part of the buffer assignment on inputs
                }
            })
            let outputs = node.outputs.map(port => { port: port.name, buffer: port.buffer as Buffer})
            
            // add this node to the input/output assignments
            order.push({ node, inputs, outputs })

            // mark the node as visited.
            node.visited = true
        }
        
        // return the output buffer assignments, relative to their corresponding input port.
        return {
            latency = node.latency as number,
            outputs = node.outputs.map((port) => {
                port: port.connection ? port.connection.port : "",
                buffer: port.buffer as Buffer
            })
        }
    }

    visit(root)
    return order
}
```

Notice we added the compensation required for an input `Port` to its buffer assignment structure. The type should be updated to reflect this.

```ts
type BufferAssignment = {
    port:String,
    buffer:Buffer,
    compensation?:number,
}
```

Implementing the actual delay is outside the scope of this discussion, so we'll just pretend we have a function that does it for us and update the rendering algorithm.

```ts
function delay (buffer:Buffer, amount:number) {
    /* ... */
}

function render (schedule:Scheduled[]) {
    for (let { node, inputs, outputs } in schedule) {
        for (let {port, buffer, compensation } in inputs) {
            if (compensation) {
                delay(buffer, compensation)
            }
        }
        node.proecss(inputs, outputs)
    }
}
```

## Further Resources

The canonical text on latency compensation is [Robin Gareus's PhD thesis](https://gareus.org/misc/thesis-p8/2017-12-Gareus-Lat.pdf) which goes much more into detail about the problems one must solve to realize latency compensation.

Dave Rowland's [talk on the Tracktion Graph library](https://github.com/drowaudio/presentations#introducing-tracktion-graph), and the the [tracktion graph source code](https://github.com/Tracktion/tracktion_engine/tree/master/modules/tracktion_graph). 

## Thanks

Special thanks to previous and ongoing discussion with wrl, ollpu, christian, the rest of the Rust Audio community, and Patrick Li. This is the umpteenth iteration of a scheduling algorithm that has continuously improved!
