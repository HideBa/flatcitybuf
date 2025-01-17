// automatically generated by the FlatBuffers compiler, do not modify

/* eslint-disable @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any, @typescript-eslint/no-non-null-assertion */

import * as flatbuffers from 'flatbuffers';

import { Vector } from '../flat-city-buf/vector.js';


export class GeographicalExtent {
  bb: flatbuffers.ByteBuffer|null = null;
  bb_pos = 0;
  __init(i:number, bb:flatbuffers.ByteBuffer):GeographicalExtent {
  this.bb_pos = i;
  this.bb = bb;
  return this;
}

min(obj?:Vector):Vector|null {
  return (obj || new Vector()).__init(this.bb_pos, this.bb!);
}

max(obj?:Vector):Vector|null {
  return (obj || new Vector()).__init(this.bb_pos + 24, this.bb!);
}

static sizeOf():number {
  return 48;
}

static createGeographicalExtent(builder:flatbuffers.Builder, min_x: number, min_y: number, min_z: number, max_x: number, max_y: number, max_z: number):flatbuffers.Offset {
  builder.prep(8, 48);
  builder.prep(8, 24);
  builder.writeFloat64(max_z);
  builder.writeFloat64(max_y);
  builder.writeFloat64(max_x);
  builder.prep(8, 24);
  builder.writeFloat64(min_z);
  builder.writeFloat64(min_y);
  builder.writeFloat64(min_x);
  return builder.offset();
}

}
