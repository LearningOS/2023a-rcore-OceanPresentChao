## 编程题
### 进程创建
实现spawn系统调用，spawn函数的作用简单来说就是创建一个子进程，并在子进程中加载执行传入的可执行文件。通过它的作用我们可以意识到它其实是fork和exec两个函数的结合，因此spawn的具体实现就可以在fork和exec函数里左借鉴右借鉴，这就是——拿来主义。

1. spawn会根据传入的文件名，解析elf文件，获取elf文件的地址空间、用户栈等。和exec类似
2. spawn会创建新的子进程，需要申请新的pid和内核栈。和fork类似
3. 根据解析exec得到的信息以及父进程的信息初始化 子进程的TCB和Trap Context。和exec、fork类似
4. 将新的进程加入队列

### stride 调度算法
这题相对比较简单，根据题目要求为TCB加入pass、priority、stride字段即可，BIG_PASS随便设置一个比较大的整数就行。
需要修改的函数就是获取下一个执行任务的函数——fetch函数，之前采用的调度策略就是先进先出。stride算法的实现根据题目要求写一个for循环寻找stride值最小的task即可。

## 问答题
### stride 算法深入

>stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

#### 实际情况是轮到 p1 执行吗？为什么？
不会，因为p2的 stride 会被更新为 `(250 + 10) as u8`,即`9`。因此下一次还是p2被调度

>我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， 在不考虑溢出的情况下 , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

#### 为什么？尝试简单说明（不要求严格证明）。
使用 8 bits 存储 stride, BigStride = 255。因此STRIDE_MAX = BigStride / min(priority) <= BigStride / 2;同时STRIDE_MIN = BigStride / max(priority) 无限接近于 0 。可得 STRIDE_MAX – STRIDE_MIN <= BigStride / 2

#### 已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。

```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.0 as i64).cmp(&(other.0 as i64)))
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

## 荣誉准则

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

[参考一](https://juejin.cn/s/linux%20spawn%20vs%20fork)

参考二：官方二阶段微信交流群

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。