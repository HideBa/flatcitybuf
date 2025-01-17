// automatically generated by the FlatBuffers compiler, do not modify

/* eslint-disable @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any, @typescript-eslint/no-non-null-assertion */

import * as flatbuffers from 'flatbuffers';

import { CityObject } from '../flat-city-buf/city-object.js';
import { Vertex } from '../flat-city-buf/vertex.js';


export class CityFeature {
  bb: flatbuffers.ByteBuffer|null = null;
  bb_pos = 0;
  __init(i:number, bb:flatbuffers.ByteBuffer):CityFeature {
  this.bb_pos = i;
  this.bb = bb;
  return this;
}

static getRootAsCityFeature(bb:flatbuffers.ByteBuffer, obj?:CityFeature):CityFeature {
  return (obj || new CityFeature()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

static getSizePrefixedRootAsCityFeature(bb:flatbuffers.ByteBuffer, obj?:CityFeature):CityFeature {
  bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
  return (obj || new CityFeature()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

id():string|null
id(optionalEncoding:flatbuffers.Encoding):string|Uint8Array|null
id(optionalEncoding?:any):string|Uint8Array|null {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? this.bb!.__string(this.bb_pos + offset, optionalEncoding) : null;
}

objects(index: number, obj?:CityObject):CityObject|null {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? (obj || new CityObject()).__init(this.bb!.__indirect(this.bb!.__vector(this.bb_pos + offset) + index * 4), this.bb!) : null;
}

objectsLength():number {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
}

vertices(index: number, obj?:Vertex):Vertex|null {
  const offset = this.bb!.__offset(this.bb_pos, 8);
  return offset ? (obj || new Vertex()).__init(this.bb!.__vector(this.bb_pos + offset) + index * 12, this.bb!) : null;
}

verticesLength():number {
  const offset = this.bb!.__offset(this.bb_pos, 8);
  return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
}

static startCityFeature(builder:flatbuffers.Builder) {
  builder.startObject(3);
}

static addId(builder:flatbuffers.Builder, idOffset:flatbuffers.Offset) {
  builder.addFieldOffset(0, idOffset, 0);
}

static addObjects(builder:flatbuffers.Builder, objectsOffset:flatbuffers.Offset) {
  builder.addFieldOffset(1, objectsOffset, 0);
}

static createObjectsVector(builder:flatbuffers.Builder, data:flatbuffers.Offset[]):flatbuffers.Offset {
  builder.startVector(4, data.length, 4);
  for (let i = data.length - 1; i >= 0; i--) {
    builder.addOffset(data[i]!);
  }
  return builder.endVector();
}

static startObjectsVector(builder:flatbuffers.Builder, numElems:number) {
  builder.startVector(4, numElems, 4);
}

static addVertices(builder:flatbuffers.Builder, verticesOffset:flatbuffers.Offset) {
  builder.addFieldOffset(2, verticesOffset, 0);
}

static startVerticesVector(builder:flatbuffers.Builder, numElems:number) {
  builder.startVector(12, numElems, 4);
}

static endCityFeature(builder:flatbuffers.Builder):flatbuffers.Offset {
  const offset = builder.endObject();
  builder.requiredField(offset, 4) // id
  return offset;
}

static finishCityFeatureBuffer(builder:flatbuffers.Builder, offset:flatbuffers.Offset) {
  builder.finish(offset);
}

static finishSizePrefixedCityFeatureBuffer(builder:flatbuffers.Builder, offset:flatbuffers.Offset) {
  builder.finish(offset, undefined, true);
}

static createCityFeature(builder:flatbuffers.Builder, idOffset:flatbuffers.Offset, objectsOffset:flatbuffers.Offset, verticesOffset:flatbuffers.Offset):flatbuffers.Offset {
  CityFeature.startCityFeature(builder);
  CityFeature.addId(builder, idOffset);
  CityFeature.addObjects(builder, objectsOffset);
  CityFeature.addVertices(builder, verticesOffset);
  return CityFeature.endCityFeature(builder);
}
}
