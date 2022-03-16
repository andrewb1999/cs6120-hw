#include "llvm/Pass.h"
#include "llvm/IR/Function.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/IR/LegacyPassManager.h"
#include "llvm/Transforms/IPO/PassManagerBuilder.h"
#include "llvm/IR/InstrTypes.h"
#include "llvm/IR/Operator.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/Support/Compiler.h"
#include <cmath>

using namespace llvm;

namespace {
  struct FastMulPass : public FunctionPass {
    static char ID;
    FastMulPass() : FunctionPass(ID) {}

    virtual bool runOnFunction(Function &F) {
      bool changed = false;
      for (auto& B : F) {
        for (auto& I : B) {
          if (auto* op = dyn_cast<MulOperator>(&I)) {
            IRBuilder<> builder(&I);
            Value* lhs = op->getOperand(0);
            Value* rhs = op->getOperand(1);


            if (auto* c = dyn_cast<ConstantInt>(rhs)) {
              if (c->getSExtValue() > 0 && isPowerOf2_64(c->getSExtValue())) {
                errs() << "Const: " << c->getSExtValue() << "\n";
                uint64_t val = Log2_64(c->getSExtValue());
                Type* t = c->getType();
                Value* new_val = ConstantInt::get(t, val, true);
                Value* shl = builder.CreateShl(lhs, new_val);

                for (auto& U : op->uses()) {
                  User* user = U.getUser();
                  user->setOperand(U.getOperandNo(), shl);
                }

                changed = true;
              }
            }
          }
        }
      }

      return changed;
    }
  };
}

char FastMulPass::ID = 0;

// Automatically enable the pass.
// http://adriansampson.net/blog/clangpass.html
static void registerSkeletonPass(const PassManagerBuilder &,
                         legacy::PassManagerBase &PM) {
  PM.add(new FastMulPass());
}
static RegisterStandardPasses
  RegisterMyPass(PassManagerBuilder::EP_EarlyAsPossible,
                 registerSkeletonPass);
