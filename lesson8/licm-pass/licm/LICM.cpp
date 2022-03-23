#include "llvm/Pass.h"
#include "llvm/IR/Function.h"
#include "llvm/Analysis/LoopPass.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/IR/LegacyPassManager.h"
#include "llvm/Transforms/IPO/PassManagerBuilder.h"
#include "llvm/IR/InstrTypes.h"
#include "llvm/IR/Operator.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/Support/Compiler.h"
#include "llvm/Transforms/Utils.h"
#include "llvm/Transforms/Scalar.h"
#include <cmath>
#include <set>

using namespace llvm;

namespace {
  struct LICMPass : public LoopPass {
    static char ID;
    LICMPass() : LoopPass(ID) {}

    virtual bool doInitialization (Loop *L, LPPassManager &LPM) {
      return false;
    }

    virtual bool runOnLoop(Loop *L, LPPassManager &LPM) {
      BasicBlock *preheader = L->getLoopPreheader();
      // L->dump();
      bool changed = false;
      std::set<Value*> loop_invariant_set;
      
      do {
        changed = false;
        for (auto* B : L->getBlocksVector()) {
          for (auto& I : *B) {
            if (!I.mayHaveSideEffects()) {
              bool loop_invariant = true;
              for (auto& o : I.operands()) {
                if (!(L->isLoopInvariant(o) || loop_invariant_set.count(o) != 0)) {
                  loop_invariant = false;
                }
              }
              if (loop_invariant && !I.isTerminator() && loop_invariant_set.count(&I) == 0
                  && !I.mayReadFromMemory() && !dyn_cast<AllocaInst>(&I) 
                  // && !dyn_cast<GetElementPtrInst>(&I)
                  ) {
                loop_invariant_set.insert(&I);
                changed = true;
              }
            }
          }
        }
      } while (changed);
      
      changed = false;
      for (Value *V : loop_invariant_set) {
        if (auto *I = dyn_cast<Instruction>(V)) {
          I->dump();
          I->moveBefore(&preheader->back());
          changed = true;
        }
      }

      return changed;
    }

    virtual bool doFinalization () {
      return false;
    }
  };
}

char LICMPass::ID = 0;

static void registerLICMPass(const PassManagerBuilder &,
                         legacy::PassManagerBase &PM) {
  // errs() << "yo\n";
  PM.add(createPromoteMemoryToRegisterPass());
  // PM.add(createLICMPass());
  PM.add(new LICMPass());
}
static RegisterStandardPasses
  RegisterMyPass(PassManagerBuilder::EP_EarlyAsPossible,
                 registerLICMPass);
