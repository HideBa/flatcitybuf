// automatically generated by the FlatBuffers compiler, do not modify

/* eslint-disable @typescript-eslint/no-unused-vars, @typescript-eslint/no-explicit-any, @typescript-eslint/no-non-null-assertion */

import * as flatbuffers from 'flatbuffers';

import { SemanticSurfaceType } from '../flat-city-buf/semantic-surface-type.js';


export class SemanticObject {
  bb: flatbuffers.ByteBuffer|null = null;
  bb_pos = 0;
  __init(i:number, bb:flatbuffers.ByteBuffer):SemanticObject {
  this.bb_pos = i;
  this.bb = bb;
  return this;
}

static getRootAsSemanticObject(bb:flatbuffers.ByteBuffer, obj?:SemanticObject):SemanticObject {
  return (obj || new SemanticObject()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

static getSizePrefixedRootAsSemanticObject(bb:flatbuffers.ByteBuffer, obj?:SemanticObject):SemanticObject {
  bb.setPosition(bb.position() + flatbuffers.SIZE_PREFIX_LENGTH);
  return (obj || new SemanticObject()).__init(bb.readInt32(bb.position()) + bb.position(), bb);
}

type():SemanticSurfaceType {
  const offset = this.bb!.__offset(this.bb_pos, 4);
  return offset ? this.bb!.readUint8(this.bb_pos + offset) : SemanticSurfaceType.RoofSurface;
}

attributes(index: number):number|null {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? this.bb!.readUint8(this.bb!.__vector(this.bb_pos + offset) + index) : 0;
}

attributesLength():number {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
}

attributesArray():Uint8Array|null {
  const offset = this.bb!.__offset(this.bb_pos, 6);
  return offset ? new Uint8Array(this.bb!.bytes().buffer, this.bb!.bytes().byteOffset + this.bb!.__vector(this.bb_pos + offset), this.bb!.__vector_len(this.bb_pos + offset)) : null;
}

children(index: number):number|null {
  const offset = this.bb!.__offset(this.bb_pos, 8);
  return offset ? this.bb!.readUint32(this.bb!.__vector(this.bb_pos + offset) + index * 4) : 0;
}

childrenLength():number {
  const offset = this.bb!.__offset(this.bb_pos, 8);
  return offset ? this.bb!.__vector_len(this.bb_pos + offset) : 0;
}

childrenArray():Uint32Array|null {
  const offset = this.bb!.__offset(this.bb_pos, 8);
  return offset ? new Uint32Array(this.bb!.bytes().buffer, this.bb!.bytes().byteOffset + this.bb!.__vector(this.bb_pos + offset), this.bb!.__vector_len(this.bb_pos + offset)) : null;
}

parent():number|null {
  const offset = this.bb!.__offset(this.bb_pos, 10);
  return offset ? this.bb!.readUint32(this.bb_pos + offset) : null;
}

static startSemanticObject(builder:flatbuffers.Builder) {
  builder.startObject(4);
}

static addType(builder:flatbuffers.Builder, type:SemanticSurfaceType) {
  builder.addFieldInt8(0, type, SemanticSurfaceType.RoofSurface);
}

static addAttributes(builder:flatbuffers.Builder, attributesOffset:flatbuffers.Offset) {
  builder.addFieldOffset(1, attributesOffset, 0);
}

static createAttributesVector(builder:flatbuffers.Builder, data:number[]|Uint8Array):flatbuffers.Offset {
  builder.startVector(1, data.length, 1);
  for (let i = data.length - 1; i >= 0; i--) {
    builder.addInt8(data[i]!);
  }
  return builder.endVector();
}

static startAttributesVector(builder:flatbuffers.Builder, numElems:number) {
  builder.startVector(1, numElems, 1);
}

static addChildren(builder:flatbuffers.Builder, childrenOffset:flatbuffers.Offset) {
  builder.addFieldOffset(2, childrenOffset, 0);
}

static createChildrenVector(builder:flatbuffers.Builder, data:number[]|Uint32Array):flatbuffers.Offset;
/**
 * @deprecated This Uint8Array overload will be removed in the future.
 */
static createChildrenVector(builder:flatbuffers.Builder, data:number[]|Uint8Array):flatbuffers.Offset;
static createChildrenVector(builder:flatbuffers.Builder, data:number[]|Uint32Array|Uint8Array):flatbuffers.Offset {
  builder.startVector(4, data.length, 4);
  for (let i = data.length - 1; i >= 0; i--) {
    builder.addInt32(data[i]!);
  }
  return builder.endVector();
}

static startChildrenVector(builder:flatbuffers.Builder, numElems:number) {
  builder.startVector(4, numElems, 4);
}

static addParent(builder:flatbuffers.Builder, parent:number) {
  builder.addFieldInt32(3, parent, null);
}

static endSemanticObject(builder:flatbuffers.Builder):flatbuffers.Offset {
  const offset = builder.endObject();
  return offset;
}

static createSemanticObject(builder:flatbuffers.Builder, type:SemanticSurfaceType, attributesOffset:flatbuffers.Offset, childrenOffset:flatbuffers.Offset, parent:number|null):flatbuffers.Offset {
  SemanticObject.startSemanticObject(builder);
  SemanticObject.addType(builder, type);
  SemanticObject.addAttributes(builder, attributesOffset);
  SemanticObject.addChildren(builder, childrenOffset);
  if (parent !== null)
    SemanticObject.addParent(builder, parent);
  return SemanticObject.endSemanticObject(builder);
}
}