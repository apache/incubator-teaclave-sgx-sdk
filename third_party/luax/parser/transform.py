import json
import sys

inAst = json.loads(sys.stdin.read())

def reprsInt(s):
    try:
        int(s)
        return True
    except ValueError:
        return False

class Node:
    def __init__(self):
        self.tag = None
        self.children = []
        self.value = None

    @staticmethod
    def fromValue(v):
        result = Node()
        result.value = v
        return result

    @staticmethod
    def fromInput(inNode):
        result = Node()

        if type(inNode) == dict:
            try:
                result.tag = inNode["tag"]
            except KeyError:
                return None
            for k in inNode:
                if reprsInt(k):
                    v = Node.fromInput(inNode[k])
                    if v != None:
                        result.children.append(v)
        elif type(inNode) == list:
            for child in inNode:
                v = Node.fromInput(child)
                if v != None:
                    result.children.append(v)
        else:
            result = Node.fromValue(inNode)
                    
        return result

    def toDict(self):
        if self.value != None:
            return self.value

        if self.tag == "Op":
            if len(self.children) == 3:
                opName = Node._mapOpName(self.children[2].value)
                return {
                    opName: [
                        self.children[0].toDict(),
                        self.children[1].toDict()
                    ]
                }
            elif len(self.children) == 2:
                opName = Node._mapOpName(self.children[0].value)
                return {
                    opName: self.children[1].toDict()
                }
            else:
                assert(False)
        elif self.tag == "NameList" or self.tag == "ExpList" or self.tag == "VarList":
            return list(map(lambda x: x.toDict(), self.children))
        elif self.tag == "Id":
            assert(len(self.children) == 1)
            return {
                "Id": self.children[0].value
            }
        elif self.tag == "Number" or self.tag == "Boolean" or self.tag == "String":
            assert(len(self.children) == 1)
            return {
                self.tag: self.children[0].value
            }
        elif self.tag == "False":
            return {
                "Boolean": False
            }
        elif self.tag == "True":
            return {
                "Boolean": True
            }
        elif self.tag == "Call":
            assert(len(self.children) >= 1)
            argList = list(map(lambda x: x.toDict(), self.children))
            del argList[0]
            return {
                "Call": [
                    self.children[0].toDict(),
                    argList
                ]
            }
        elif self.tag == "Fornum":
            # (ident, expr, expr, block)
            if len(self.children) == 4:
                return {
                    "Fornum": [
                        self.children[0].toDict(),
                        self.children[1].toDict(),
                        self.children[2].toDict(),
                        None,
                        self.children[3].toDict()
                    ]
                }
            # (ident, expr, expr, expr, block)
            elif len(self.children) == 5:
                # The normal path
                pass
            else:
                assert(False)
        elif self.tag == "Function":
            if len(self.children) == 1:
                return {
                    "Function": [
                        [],
                        self.children[0].toDict()
                    ]
                }
        elif self.tag == "If":
            branchList = []
            for i in range(0, int(len(self.children) / 2)):
                branchList.append([
                    self.children[i * 2].toDict(),
                    self.children[i * 2 + 1].toDict()
                ])
            elseBranch = None
            if len(self.children) % 2 == 1:
                elseBranch = self.children[len(self.children) - 1].toDict()
            return {
                "If": [
                    branchList,
                    elseBranch
                ]
            }
        elif self.tag == "Break":
            return "Break"
        elif self.tag == "Paren": # Is this correct?
            assert(len(self.children) == 1)
            return self.children[0].toDict()

        v = list(map(lambda x: x.toDict(), self.children))

        if self.tag == None:
            return v
        else:
            return {
                self.tag: v
            }

    @staticmethod
    def _mapOpName(name):
        opTable = {
            "add": "Add",
            "sub": "Sub",
            "mul": "Mul",
            "div": "Div",
            "mod": "Mod",
            "pow": "Pow",
            "eq": "Eq",
            "ne": "Ne",
            "lt": "Lt",
            "gt": "Gt",
            "le": "Le",
            "ge": "Ge",
            "not": "Not",
            "unm": "Unm",
            "concat": "Concat",
            "and": "And",
            "or": "Or"
        }
        if name in opTable:
            return opTable[name]
        else:
            raise Exception("Unknown op name: " + name)

print(json.dumps(Node.fromInput(inAst).toDict()))
