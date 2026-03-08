#include <cstddef>
#include <cstdint>
#include <limits>
#include <memory>
#include <vector>

#include "ymfm.h"
#include "ymfm_opn.h"

namespace {

class Ym2612Interface final : public ymfm::ymfm_interface {
public:
    Ym2612Interface() { reset(); }

    void reset() {
        current_clock = 0;
        busy_end_clock = 0;
        irq_asserted = false;
        timer_end_clock[0] = kDisabledTimer;
        timer_end_clock[1] = kDisabledTimer;
    }

    void advance(uint32_t clocks) {
        uint64_t target_clock = current_clock + clocks;
        while (true) {
            int timer_index = -1;
            uint64_t next_timer_clock = target_clock + 1;
            for (int index = 0; index < 2; index++) {
                if (timer_end_clock[index] <= target_clock &&
                    timer_end_clock[index] < next_timer_clock) {
                    timer_index = index;
                    next_timer_clock = timer_end_clock[index];
                }
            }
            if (timer_index < 0) {
                break;
            }

            current_clock = next_timer_clock;
            timer_end_clock[timer_index] = kDisabledTimer;
            m_engine->engine_timer_expired(static_cast<uint32_t>(timer_index));
        }

        current_clock = target_clock;
    }

    bool irq_state() const { return irq_asserted; }

    void ymfm_set_timer(uint32_t tnum, int32_t duration_in_clocks) override {
        if (tnum >= 2) {
            return;
        }
        if (duration_in_clocks < 0) {
            timer_end_clock[tnum] = kDisabledTimer;
            return;
        }

        timer_end_clock[tnum] = current_clock + static_cast<uint64_t>(duration_in_clocks);
    }

    void ymfm_set_busy_end(uint32_t clocks) override {
        busy_end_clock = current_clock + clocks;
    }

    bool ymfm_is_busy() override {
        return current_clock < busy_end_clock;
    }

    void ymfm_update_irq(bool asserted) override {
        irq_asserted = asserted;
    }

private:
    static constexpr uint64_t kDisabledTimer = std::numeric_limits<uint64_t>::max();

    uint64_t current_clock = 0;
    uint64_t busy_end_clock = 0;
    uint64_t timer_end_clock[2] = {kDisabledTimer, kDisabledTimer};
    bool irq_asserted = false;
};

struct Ym2612Handle {
    Ym2612Interface interface;
    ymfm::ym2612 chip;

    Ym2612Handle() : interface(), chip(interface) {
        interface.reset();
        chip.reset();
    }
};

}  // namespace

extern "C" {

void* ymfm_ym2612_create() {
    return new Ym2612Handle();
}

void ymfm_ym2612_destroy(void* chip) {
    delete static_cast<Ym2612Handle*>(chip);
}

void ymfm_ym2612_reset(void* chip) {
    auto* handle = static_cast<Ym2612Handle*>(chip);
    handle->interface.reset();
    handle->chip.reset();
}

uint32_t ymfm_ym2612_sample_rate(uint32_t input_clock_hz) {
    Ym2612Interface interface;
    ymfm::ym2612 chip(interface);
    return chip.sample_rate(input_clock_hz);
}

uint8_t ymfm_ym2612_read(void* chip, uint8_t port) {
    auto& ym = static_cast<Ym2612Handle*>(chip)->chip;
    return ym.read(port & 0x03);
}

void ymfm_ym2612_advance(void* chip, uint32_t clocks) {
    static_cast<Ym2612Handle*>(chip)->interface.advance(clocks);
}

bool ymfm_ym2612_irq_asserted(void* chip) {
    return static_cast<Ym2612Handle*>(chip)->interface.irq_state();
}

void ymfm_ym2612_write(void* chip, uint8_t port, uint8_t value) {
    auto& ym = static_cast<Ym2612Handle*>(chip)->chip;
    switch (port & 0x03) {
        case 0x00:
            ym.write_address(value);
            break;
        case 0x01:
            ym.write_data(value);
            break;
        case 0x02:
            ym.write_address_hi(value);
            break;
        case 0x03:
            ym.write_data_hi(value);
            break;
    }
}

void ymfm_ym2612_generate(void* chip, int32_t* left, int32_t* right, std::size_t sample_count) {
    auto& ym = static_cast<Ym2612Handle*>(chip)->chip;
    std::vector<ymfm::ym2612::output_data> output(sample_count);
    ym.generate(output.data(), static_cast<uint32_t>(sample_count));
    for (std::size_t index = 0; index < sample_count; index++) {
        output[index].clamp16();
        left[index] = output[index].data[0];
        right[index] = output[index].data[1];
    }
}

}  // extern "C"
