
/******************************************************************************
audio-graph-prototype: demonstration of a node scheduling algorithm.
Copyright (C) 2021  Michael Hilgendorf

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
******************************************************************************/

export {}
// We don't care about what a buffer looks like internally
type Buffer = {
  /* ... */
}

type BufferAssignment = {
  port: string //
  buffer: Buffer
  compensation?: number
}

type Port = {
  name: string
  connection?: {
    node: Node
    port: string
  }
  latency?: number
  compensation?: number
  buffer?: Buffer
}

type Node = {
  name: string
  inputs: Port[]
  outputs: Port[]
  latency?: number
  delay: number
  visited: boolean
  process: (inputs: BufferAssignment[], outputs: BufferAssignment[]) => void
}

type Scheduled = {
  node: Node
  inputs: BufferAssignment[]
  outputs: BufferAssignment[]
}

function delay(buffer: Buffer, amount: number) {
  console.log(`delaying ${buffer} by ${amount}`)
  /* ... */
}

function render(schedule: Scheduled[]) {
  for (let { node, inputs, outputs } of schedule) {
    for (let { buffer, compensation } of inputs) {
      if (compensation) {
        delay(buffer, compensation)
      }
    }
    node.process(inputs, outputs)
  }
}

function schedule(root: Node): Scheduled[] {
  let order: Scheduled[] = []
  let bufferStack: Buffer[] = []
  function acquireBuffer(): Buffer {
    if (bufferStack.length === 0) {
      return {}
    } else {
      return bufferStack.pop() as Buffer
    }
  }
  function releaseBuffer(buffer: Buffer) {
    bufferStack.push(buffer)
  }
  function visit(node: Node): { latency: number; outputs: BufferAssignment[] } {
    if (!node.visited) {
      // for each output in this port, acquire a new buffer
      for (let output of node.outputs) {
        output.buffer = acquireBuffer()
      }

      // for each input port, find the input buffer and solve for its latency
      for (let input of node.inputs) {
        // if the input port is connected, solve the node on the other side
        // and find the corresponding output buffer
        if (input.connection) {
          let { latency, outputs } = visit(input.connection.node)
          let correspondingBuffer = outputs.find(
            (assn) => assn.port === input.name
          )
          input.latency = latency
          input.buffer = correspondingBuffer?.buffer
        } else {
          // if there is no connection, acquire a new buffer and set the latency.
          input.latency = 0
          input.buffer = acquireBuffer()
          // since this buffer will not be reused, we can release it immediately.
          releaseBuffer(input.buffer as Buffer)
        }
      }

      // release buffers
      for (let output of node.outputs) {
        releaseBuffer(output.buffer as Buffer)
      }

      // compute max latency
      let maxInputLatency = node.inputs
        .map((port) => port.latency as number)
        .reduce((prev, curr) => Math.max(prev, curr), 0)

      // compute input compensations
      for (let input of node.inputs) {
        let compensation = maxInputLatency - (input.latency as number)
        if (compensation != 0) {
          input.compensation = compensation
        }
      }

      // update the latency of this node
      node.latency = maxInputLatency + node.delay

      // collect input and output buffer assignments
      let inputs = node.inputs.map((port) => {
        return {
          port: port.name,
          buffer: port.buffer as Buffer,
          compensation: port.compensation,
        }
      })
      let outputs = node.outputs.map((port) => {
        return { port: port.name, buffer: port.buffer as Buffer }
      })

      // add this node and its input/output buffer assignments to the order
      order.push({ node, inputs, outputs })

      // mark the node as visited
      node.visited = true
    }

    // finally, return the latency of this node and the output buffer assignments -
    // relative to their corresponding input port.
    return {
      latency: node.latency as number,
      outputs: node.outputs.map((port) => {
        return {
          // this will never be used if the connection doesn't exist
          port: port.connection ? port.connection.port : "",
          buffer: port.buffer as Buffer,
        }
      }),
    }
  }
  visit(root)
  return order
}

/*****************************************************************************/
/**************************** Graph Example &*********************************/
/*****************************************************************************/
let source: Node = {
  name: "source",
  process: (i, o) => console.log("called Source"),
  delay: 0,
  inputs: [],
  outputs: [{ name: "out1" }, { name: "out2" }],
  visited: false,
}

let left: Node = {
  name: "Left",
  process: (i, o) => console.log("called Left"),
  delay: 1,
  inputs: [{ name: "in1" }],
  outputs: [{ name: "out1" }],
  visited: false,
}

let right: Node = {
  name: "Right",
  process: (i, o) => console.log("called Right"),
  delay: 2,
  inputs: [{ name: "in1" }],
  outputs: [{ name: "out1" }],
  visited: false,
}

let sink: Node = {
  name: "Sink",
  process: (i, o) => console.log("called Sink"),
  delay: 0,
  inputs: [{ name: "in1" }, { name: "in2" }],
  outputs: [],
  visited: false,
}

source.outputs[0].connection = { node: left, port: "in1" }
source.outputs[1].connection = { node: right, port: "in1" }
left.inputs[0].connection = { node: source, port: "out1" }
right.inputs[0].connection = { node: source, port: "out2" }
left.outputs[0].connection = { node: sink, port: "in1" }
right.outputs[0].connection = { node: sink, port: "in2" }
sink.inputs[0].connection = { node: left, port: "out1" }
sink.inputs[1].connection = { node: right, port: "out2" }

render(schedule(sink))
