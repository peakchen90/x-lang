#include "llvm/ADT/APFloat.h"
#include "llvm/ADT/STLExtras.h"
#include "llvm/IR/BasicBlock.h"
#include "llvm/IR/Constants.h"
#include "llvm/IR/DerivedTypes.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Type.h"
#include "llvm/IR/Verifier.h"

#include <iostream>

using namespace llvm;
using namespace std;

static LLVMContext context;
static IRBuilder<> Builder(context);
static unique_ptr<Module> module;
static map<std::string, Value *> nameMap;

extern "C" Value get_const_number(double num)
{
  return ConstantFP::get(context, APFloat(num));
}