# automatically generated by the FlatBuffers compiler, do not modify

# namespace: FlatCityBuf

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class Vector(object):
    __slots__ = ['_tab']

    @classmethod
    def SizeOf(cls):
        return 24

    # Vector
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # Vector
    def X(self): return self._tab.Get(flatbuffers.number_types.Float64Flags, self._tab.Pos + flatbuffers.number_types.UOffsetTFlags.py_type(0))
    # Vector
    def Y(self): return self._tab.Get(flatbuffers.number_types.Float64Flags, self._tab.Pos + flatbuffers.number_types.UOffsetTFlags.py_type(8))
    # Vector
    def Z(self): return self._tab.Get(flatbuffers.number_types.Float64Flags, self._tab.Pos + flatbuffers.number_types.UOffsetTFlags.py_type(16))

def CreateVector(builder, x, y, z):
    builder.Prep(8, 24)
    builder.PrependFloat64(z)
    builder.PrependFloat64(y)
    builder.PrependFloat64(x)
    return builder.Offset()