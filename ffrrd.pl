#!/usr/bin/perl

use strict;
use warnings;
use Data::Dumper;
use feature 'say';
use RRDTool::OO;
use LWP::Simple;
use JSON::XS;
use constant {
	RRD_SYSTEM    => "system.rrd",
	RRD_NETDEV    => "netdev_{}.rrd",

	RASPBERRY => 'ff6188',
	APPLE => 'a9dc76',
	LEMON => 'ffd866',
	ORANGE => 'fc9867',
	LAVENDER => 'ab9df2',
	BLUE => '78dce8',
};


my @ARCHIVE = (
	archive => {
		rows => 2880,
		cfunc => 'AVERAGE'
	},
	archive => {
		rows => 20160,
		cfunc => 'AVERAGE',
		cpoints => 15,
	},
	archive => {
		rows => 259200,
		cfunc => 'AVERAGE',
		cpoints => '60'
	}
);


package SysStats {
	use Data::Dumper;


	sub new() {
		my $class = shift;

		my $self = {};

		$self->{'mem'} = get_meminfo();
		$self->{'cpu'} = get_stat();
		$self->{'load'} = get_loadavg();
		$self->{'vm'} = get_vmstat();

		return bless $self, $class;
	}


	sub get_systat() {
		my $self = shift;

		my $sysstat = {
			cpu_user       => $self->{cpu}->{user},
			cpu_nice       => $self->{cpu}->{nice},
			cpu_system     => $self->{cpu}->{system},
			cpu_idle       => $self->{cpu}->{idle},
			cpu_iowait     => $self->{cpu}->{iowait},
			cpu_irq        => $self->{cpu}->{irq},
			cpu_softirq    => $self->{cpu}->{softirq},
			cpu_steal      => $self->{cpu}->{steal},
			cpu_guest 	   => $self->{cpu}->{guest},
			cpu_guest_nice => $self->{cpu}->{guest_nice},

			mem_free       => $self->{mem}->{MemFree},
			mem_total      => $self->{mem}->{MemTotal},
			mem_available  => $self->{mem}->{MemAvailable},
			mem_shared     => $self->{mem}->{Shmem},
			mem_buffered   => $self->{mem}->{Buffers},
			mem_cached     => $self->{mem}->{Cached},
			mem_swap_free  => $self->{mem}->{SwapFree},

			load_1min      => $self->{load}->{'1min'},
			load_5min      => $self->{load}->{'5min'},
			load_15min     => $self->{load}->{'15min'},

			procs_num      => $self->{load}->{processes},
		};

		foreach(keys(%$sysstat)) {

			#die "undef key: $_" if !$sysstat->{$_};
		}

		return $sysstat;
	}


	sub get_stat() {
		open(STAT, "<", "/proc/stat") or die $!;
		while (my $line = <STAT>) {
			chomp($line);

			if($line =~ m/^cpu\s+/) {
				my @params = ($line =~ m/cpu\s+(\d+)\s(\d+)\s(\d+)\s(\d+)\s(\d+)\s(\d+)\s(\d+)\s(\d+)\s(\d+)\s(\d+)/);
				my @keys = ('user','nice','system','idle','iowait','irq','softirq','steal','guest','guest_nice');

				my %stat;
				@stat{@keys} = @params;

				return \%stat;
			}
		}
	}


	sub get_meminfo() {
		open(STAT, "<", "/proc/meminfo") or die $!;
		my $meminfo = {};

		while (my $line = <STAT>) {
			chomp($line);

			if(my ($key, $value) = ($line =~ m/^([\w\(\)]+):\s+(\d+)/)) {
				$meminfo->{$key} = $value;
			}
		}

		return $meminfo;
	}


	sub get_vmstat() {
		open(STAT, "<", "/proc/vmstat") or die $!;
		my $vmstat = {};

		while (my $line = <STAT>) {
			chomp($line);

			if(my ($key, $value) = ($line =~ m/^(\w+)\s+(\d+)$/)) {
				$vmstat->{$key} = $value;
			}
		}

		return $vmstat;
	}


	sub get_loadavg() {
		open(STAT, "<", "/proc/loadavg") or die $!;
		my $load;
		my $line = <STAT>;
		chomp($line);

		if ($line =~ m/^(\d+\.\d+)\s+(\d+\.\d+)\s+(\d+\.\d+)\s(\d+)\/(\d+).*$/) {
			$load->{'1min'} = $1;
			$load->{'5min'} = $2;
			$load->{'15min'} = $3;
			$load->{'running'} = $4;
			$load->{'processes'} = $5;
		}

		return $load;
	}
}


sub update_sysstats($$) {
	my $rrd = shift;
	my $stats = shift;

	$rrd->update(values => $stats) or die $!;
}





if(@ARGV < 1) {
	die "not enough arguments";
}






my $rrdsystem = RRDTool::OO->new(file => RRD_SYSTEM);

if(!-e RRD_SYSTEM) {
	$rrdsystem->create(
		step => 60,
		data_source => {
			name => 'cpu_user',
			type => 'COUNTER',
		},
		data_source => {
			name => 'cpu_nice',
			type => 'COUNTER',
		},
		data_source => {
			name => 'cpu_system',
			type => 'COUNTER',
		},
		data_source => {
			name => 'cpu_idle',
			type => 'COUNTER',
		},
		data_source => {
			name => 'cpu_iowait',
			type => 'COUNTER',
		},
		data_source => {
			name => 'cpu_irq',
			type => 'COUNTER',
		},
		data_source => {
			name => 'cpu_softirq',
			type => 'COUNTER',
		},
		data_source => {
			name => 'cpu_steal',
			type => 'COUNTER',
		},
		data_source => {
			name => 'cpu_guest',
			type => 'COUNTER',
		},
		data_source => {
			name => 'cpu_guest_nice',
			type => 'COUNTER',
		},
		data_source => {
			name => 'mem_total',
			type => 'GAUGE',
		},
		data_source => {
			name => 'mem_free',
			type => 'GAUGE',
		},
		data_source => {
			name => 'mem_available',
			type => 'GAUGE',
		},
		data_source => {
			name => 'mem_buffered',
			type => 'GAUGE',
		},
		data_source => {
			name => 'mem_cached',
			type => 'GAUGE',
		},
		data_source => {
			name => 'mem_swap_free',
			type => 'GAUGE',
		},
		data_source => {
			name => 'mem_shared',
			type => 'GAUGE',
		},
		data_source => {
			name => 'load_1min',
			type => 'GAUGE',
		},
		data_source => {
			name => 'load_5min',
			type => 'GAUGE',
		},
		data_source => {
			name => 'load_15min',
			type => 'GAUGE',
		},
		data_source => {
			name => 'procs_num',
			type => 'GAUGE',
		},
		@ARCHIVE,
	);
}




sub draw_netdev_graph($$$) {
	my $interface = shift or die "no device given";
	my $rrd = shift or die "no database provided";
	my $duration = shift // 6;


	#die "only full hours as durations" if($duration % 1 != 0);


	$rrd->option_add("graph", 'border');
	$rrd->option_add("graph", 'slope-mode');
	$rrd->graph(
		image => $ARGV[0]."/netdev_${interface}_${duration}.png",
		vertical_label => 'Bytes/s',
		start => time() - $duration * 3600,
		width => 1200,
		height => 300,

		# describtion => "$duration hours",
		color => {
			back => '#191919',

			#arrow => '#ff0000'
			canvas => '#111111',
			font => '#aaaaaa',
			mgrid => '#fff5',
			grid => '#fff5',
			frame => '#fff',
		},
		no_minor => undef,

		# x_grid => 'MINUTE:5:MINUTE:5:HOUR:4:0:%X',
		border => 0,
		zoom => 2,

		# font => {
		# 	name => 'monospace',
		# 	size => 15
		# },

		# hidden draws

		draw => {
			type => 'hidden',
			dsname => 'tx_bytes',
			name => 'tx_bytes',
		},
		draw => {
			type => 'hidden',
			dsname => 'rx_bytes',
			name => 'rx_bytes',
		},

		# TX
		draw => {
			type => 'area',
			dsname => 'tx_bytes',
			thickness => 2,
			color => '00cf6e33',
			cdef => "tx_bytes,8,*",
			legend => 'TX bps',
		},
		draw => {
			type => 'line',
			dsname => 'tx_bytes',
			thickness => 2,
			color => '00cf6eaa',
			cdef => "tx_bytes,".($duration*3600/18).",TRENDNAN,8,*",
			legend => 'TX bps TREND',
		},

		# RX
		draw => {
			type => 'area',
			dsname => 'rx_bytes',
			thickness => 0,
			color => 'cf006033',
			cdef => "rx_bytes,8,*,-1,*",
			legend => 'TX bps',
		},
		draw => {
			type => 'line',
			dsname => 'rx_bytes',
			thickness => 2,
			color => 'cf0060aa',
			cdef => "rx_bytes,".($duration*3600/18).",TRENDNAN,8,*,-1,*",
			legend => 'RX bps TREND',
		},
	);
}





my $stats = SysStats->new();
update_sysstats($rrdsystem, $stats->get_systat());






my $trend_window = 60*2;

$rrdsystem->option_add("graph", 'border');
$rrdsystem->graph(
	image => $ARGV[0]."/load.png",
	vertical_label => 'Load',
	start => time() - 60*60*12,
	width => 1200,
	height => 300,
	color => {
		back => '#191919',

		#arrow => '#ff0000'
		canvas => '#111111',
		font => '#aaaaaa',
		mgrid => '#fff5',
		grid => '#fff5',
		frame => '#fff',
	},
	no_minor => undef,

	# x_grid => 'MINUTE:5:MINUTE:5:HOUR:4:0:%X',
	border => 0,
	zoom => 2,

	# font => {
	# 	name => 'monospace',
	# 	size => 15
	# },
	draw => {
		type => 'hidden',
		dsname => 'load_15min',
		name => 'load15min',
	},
	draw => {
		type => 'hidden',
		dsname => 'load_1min',
		name => 'load1min',
	},
	draw => {
		type => 'area',
		dsname => 'load_1min',
		thickness => 1,
		color => ORANGE.'22',
		legend => 'Load 1 min',
		cdef => "load1min,". $trend_window .",TREND"
	},
	draw => {
		type => 'line',
		dsname => 'load_1min',
		thickness => 1,
		color => ORANGE,
		legend => 'Load 1 min',
		cdef => "load1min,". $trend_window .",TREND"
	},
	draw => {
		type => 'area',
		dsname => 'load_15min',
		thickness => 2,
		color => RASPBERRY.'66',
		legend => 'Load 15 min',
		cdef => "load15min,". $trend_window .",TREND"
	},
	draw => {
		dsname => 'load_15min',
		thickness => 2,
		color => RASPBERRY,
		legend => 'Load 15 min',
		name => 'load_15min',
		cdef => "load15min,". $trend_window .",TREND"
	},
);


# Graph memory usage
$rrdsystem->graph(
	image => $ARGV[0]."/memory.png",
	vertical_label => 'Memory',
	start => time() - 60*60*12,
	width => 1200,
	height => 300,
	color => {
		back => '#191919',

		#arrow => '#ff0000'
		canvas => '#111111',
		font => '#aaaaaa',
		mgrid => '#fff5',
		grid => '#fff5',
		frame => '#fff',
	},
	no_minor => undef,

	# x_grid => 'MINUTE:5:MINUTE:5:HOUR:4:0:%X',
	border => 0,
	zoom => 2,

	# font => {
	# 	name => 'monospace',
	# 	size => 15
	# },
	draw => {
		type => 'hidden',
		dsname => 'mem_free',
		name => 'free',
	},
	draw => {
		type => 'hidden',
		dsname => 'mem_available',
		name => 'available',
	},
	draw => {
		type => 'hidden',
		dsname => 'mem_buffered',
		name => 'buffered',
	},
	draw => {
		type => 'hidden',
		dsname => 'mem_shared',
		name => 'shared',
	},
	draw => {
		type => 'hidden',
		dsname => 'mem_total',
		name => 'total',
	},
	draw => {
		type => 'hidden',
		dsname => 'mem_cached',
		name => 'cached',
	},


	draw => {
		cdef => 'total,free,-,1000,*',
		type => 'area',
		thickness => 1,
		color => RASPBERRY.'33',
		legend => 'Used',
	},

	draw => {
		cdef => 'total,free,-,1000,*',
		type => 'line',
		thickness => 1,
		color => RASPBERRY.'99',
		legend => 'Used',
	},
	draw => {
		cdef => 'shared,1000,*',
		type => 'line',
		thickness => 1,
		color => BLUE,
		legend => 'Shared',
	},
	draw => {
		cdef => 'total,1000,*',
		type => 'line',
		thickness => 2,
		color => LEMON,
		legend => 'Total',
	},
	draw => {
		cdef => "available,1000,*",
		thickness => 1,
		color => ORANGE,
		legend => 'Available',
		name => 'Available',
	},

	draw => {
		cdef => "cached,1000,*",
		type => 'line',
		thickness => 2,
		color => LAVENDER,
		legend => 'Free',
	},

	# Stack
);


foreach my $dev (get_netdevs()) {
	my $rrdfile = RRD_NETDEV =~ s/\{\}/$dev->{interface}/r;
	my $rrd = RRDTool::OO->new(file => $rrdfile);

	if(!-e $rrdfile) {
		$rrd->create(
			step => 60,
			data_source => {
				name => 'tx_compressed',
				type => 'COUNTER',
			},
			data_source => {
				name => 'tx_packets',
				type => 'COUNTER',
			},
			data_source => {
				name => 'tx_carrier',
				type => 'COUNTER',
			},
			data_source => {
				name => 'tx_errs',
				type => 'COUNTER',
			},
			data_source => {
				name => 'rx_compressed',
				type => 'COUNTER',
			},
			data_source => {
				name => 'rx_errs',
				type => 'COUNTER',
			},
			data_source => {
				name => 'rx_packets',
				type => 'COUNTER',
			},
			data_source => {
				name => 'tx_drop',
				type => 'COUNTER',
			},
			data_source => {
				name => 'rx_multicast',
				type => 'COUNTER',
			},
			data_source => {
				name => 'tx_colls',
				type => 'COUNTER',
			},
			data_source => {
				name => 'rx_frame',
				type => 'COUNTER',
			},
			data_source => {
				name => 'tx_fifo',
				type => 'COUNTER',
			},
			data_source => {
				name => 'tx_bytes',
				type => 'COUNTER',
			},
			data_source => {
				name => 'rx_drop',
				type => 'COUNTER',
			},
			data_source => {
				name => 'rx_bytes',
				type => 'COUNTER',
			},
			data_source => {
				name => 'rx_fifo',
				type => 'COUNTER',
			},
			@ARCHIVE,
		) or die $!;
	}

	say "updating interface ".$dev->{interface};
	$rrd->update(values => $dev->{stats}) or die $!;


	my @durations = (1,3,12,96);

	foreach (@durations) {
		draw_netdev_graph($dev->{interface}, $rrd, $_);
	}

}

# Colorscheme:
# cf0060
# c8cf00
# 00cf6e
# 0700cf
