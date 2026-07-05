// AudioWorklet consumer for the wasm engine's GEMS samples (#20).
//
// SPSC ring over SharedArrayBuffer: main.js writes each stepped frame's
// interleaved-stereo f32 samples, this processor drains them at the device
// rate. Indices live in a shared Int32Array ([0]=read, [1]=write, in f32
// units, always stereo-pair aligned) so both sides move them with Atomics and
// never block — on underrun (rAF stall, background tab) the output is
// silence, never a stale loop.
class WildsideAudioProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    const { ringBuffer, stateBuffer } = options.processorOptions;
    this.ring = new Float32Array(ringBuffer);
    this.state = new Int32Array(stateBuffer);
  }

  process(_inputs, outputs) {
    const output = outputs[0];
    const left = output[0];
    const right = output[1] ?? output[0];
    const capacity = this.ring.length;
    let read = Atomics.load(this.state, 0);
    const write = Atomics.load(this.state, 1);
    let available = (write - read + capacity) % capacity;
    for (let i = 0; i < left.length; i++) {
      if (available >= 2) {
        left[i] = this.ring[read];
        right[i] = this.ring[(read + 1) % capacity];
        read = (read + 2) % capacity;
        available -= 2;
      } else {
        left[i] = 0;
        right[i] = 0;
      }
    }
    Atomics.store(this.state, 0, read);
    return true;
  }
}

registerProcessor("wildside-audio", WildsideAudioProcessor);
