# automatically generated by the FlatBuffers compiler, do not modify

# namespace: FlatCityBuf

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class ReferenceSystem(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = ReferenceSystem()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsReferenceSystem(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # ReferenceSystem
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # ReferenceSystem
    def Authority(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # ReferenceSystem
    def Version(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # ReferenceSystem
    def Code(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(8))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # ReferenceSystem
    def CodeString(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(10))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

def ReferenceSystemStart(builder):
    builder.StartObject(4)

def Start(builder):
    ReferenceSystemStart(builder)

def ReferenceSystemAddAuthority(builder, authority):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(authority), 0)

def AddAuthority(builder, authority):
    ReferenceSystemAddAuthority(builder, authority)

def ReferenceSystemAddVersion(builder, version):
    builder.PrependInt32Slot(1, version, 0)

def AddVersion(builder, version):
    ReferenceSystemAddVersion(builder, version)

def ReferenceSystemAddCode(builder, code):
    builder.PrependInt32Slot(2, code, 0)

def AddCode(builder, code):
    ReferenceSystemAddCode(builder, code)

def ReferenceSystemAddCodeString(builder, codeString):
    builder.PrependUOffsetTRelativeSlot(3, flatbuffers.number_types.UOffsetTFlags.py_type(codeString), 0)

def AddCodeString(builder, codeString):
    ReferenceSystemAddCodeString(builder, codeString)

def ReferenceSystemEnd(builder):
    return builder.EndObject()

def End(builder):
    return ReferenceSystemEnd(builder)
