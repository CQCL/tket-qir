import json
from pytket.circuit import Circuit, OpType, CircBox
from pytket.circuit import if_bit, if_not_bit

c = Circuit(2)
b = c.add_c_register("c", 2)
c.X(0).Measure(0,0)
c.Z(1, condition=if_not_bit(b[0]))

c2 = Circuit(2).CX(0,1)

c.add_circbox(CircBox(c2), [0,1], condition=b[1])


print(json.dumps(c.to_dict()))