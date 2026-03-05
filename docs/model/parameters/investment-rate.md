# Investment Rate

The fraction of industrial output allocated to gross investment in new industrial capital. In World3, this is a residual: whatever remains after consumption, services, and agriculture have claimed their shares.

**Sector:** [Capital](../sectors/capital.md)
**Source code:** `crates/world3-core/src/model/sectors/capital.rs`, computed as `frac_to_investment`
**BAU value:** Derived (residual = 1 - FIOAC - FIOAS - FIOAA), typically ~0.43 at 1900

## Equation Context

The investment rate enters the industrial capital stock equation:

$$\text{FIOAI} = \max(0,\; 1 - \text{FIOAC} - \text{FIOAS} - \text{FIOAA})$$

$$\frac{d(\text{IC})}{dt} = \text{IO} \times \text{FIOAI} - \text{IC} \times \delta$$

Where:
- **IO** = industrial output
- **FIOAC** = fraction to consumption (lookup on IOPC)
- **FIOAS** = fraction to services (lookup on SOPC/ISOPC)
- **FIOAA** = fraction to agriculture (lookup on food ratio)
- **IC** = industrial capital stock
- $\delta$ = industrial depreciation rate (1/14 yr^-1)

Because FIOAI is a residual, it can be squeezed to zero when the three allocation fractions sum to 1.0 or more. The code clamps FIOAI to a minimum of 0 — negative investment is impossible.

## Calibration

This is not a tunable parameter but a derived quantity. Its value depends on the three allocation lookup tables (FIOAC, FIOAS, FIOAA) and their inputs. In the early simulation (1900), FIOAI is approximately 0.43. During collapse, the competing fractions grow and squeeze investment toward zero, which starves capital accumulation and triggers the industrial decline feedback loop.

## Info Panel

**Unit:** fraction

**Beginner:** What fraction of industrial output is reinvested in building new capital. Higher = faster growth but less available for services and agriculture.

**Expert:** Fraction of industrial_output allocated to gross investment. d(IC)/dt investment term = IO x investment_rate.

**Feedback loops:** resource-collapse

**Related variables:** capital.industrial_capital, capital.industrial_output, capital.service_output_per_capita

**Impact increase:** Faster capital growth but less output for services -- trade-off between growth and welfare

**Impact decrease:** Slower capital growth but more services available -- better short-term welfare

**Sparkline variable:** capital.industrial_capital

## References

- Meadows et al. (2004), *Limits to Growth: The 30-Year Update*, Ch. 4 — capital sector allocation
- World3-03 Vensim: FIOAI = 1 - FIOAC - FIOAS - FIOAA (residual investment fraction)
